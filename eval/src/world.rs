use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::ops::Range;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Datelike, FixedOffset, Local};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Dict};
use typst::syntax::package::PackageSpec;
use typst::syntax::{FileId, Source, Span, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_kit::download::{Downloader, ProgressSink};
use typst_kit::fonts::{FontSearcher, FontSlot};
use typst_kit::package::PackageStorage;

/// Host environment shared by all evaluations in a runtime.
/// Fonts are discovered when the runtime is created.
#[derive(Clone, Debug, Default)]
pub struct WorldOptions {
    pub font_paths: Vec<PathBuf>,
    pub ignore_system_fonts: bool,
    pub package_path: Option<PathBuf>,
    pub package_cache_path: Option<PathBuf>,
}

/// Files and configuration retained across Typst evaluations.
pub struct ProjectWorld {
    root: PathBuf,
    main: FileId,
    inputs: Dict,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<FontSlot>,
    slots: Mutex<HashMap<FileId, FileSlot>>,
    overrides: HashMap<FileId, Bytes>,
    packages: PackageStorage,
    package_roots: Mutex<HashMap<PackageSpec, PathBuf>>,
    now: DateTime<Local>,
}

impl ProjectWorld {
    pub fn new(root: impl AsRef<Path>, entry: impl AsRef<Path>, inputs: Dict) -> Result<Self> {
        Self::new_with_options(root, entry, inputs, WorldOptions::default())
    }

    pub fn new_with_options(
        root: impl AsRef<Path>,
        entry: impl AsRef<Path>,
        inputs: Dict,
        options: WorldOptions,
    ) -> Result<Self> {
        let root = root.as_ref().canonicalize().with_context(|| {
            format!("failed to resolve project root {}", root.as_ref().display())
        })?;
        if !root.is_dir() {
            bail!("project root must be a directory");
        }
        let main = entry_id(entry.as_ref())?;
        let fonts = FontSearcher::new()
            .include_system_fonts(!options.ignore_system_fonts)
            .search_with(&options.font_paths);
        let packages = PackageStorage::new(
            options.package_cache_path,
            options.package_path,
            Downloader::new(concat!("zettyp-eval/", env!("CARGO_PKG_VERSION"))),
        );
        let library = Library::builder().with_inputs(inputs.clone()).build();

        Ok(Self {
            root,
            main,
            inputs,
            library: LazyHash::new(library),
            book: LazyHash::new(fonts.book),
            fonts: fonts.fonts,
            slots: Mutex::new(HashMap::new()),
            overrides: HashMap::new(),
            packages,
            package_roots: Mutex::new(HashMap::new()),
            now: Local::now(),
        })
    }

    /// Begin another evaluation. This does not discard unchanged sources.
    ///
    /// Files are read once on their first access in the new evaluation, including
    /// files that previously failed to load. Inputs replace, rather than merge
    /// with, the previous inputs. Invalid entries leave the world unchanged.
    pub fn prepare(&mut self, entry: impl AsRef<Path>, inputs: Dict) -> Result<()> {
        self.prepare_with_sources(entry, inputs, BTreeMap::new())
    }

    /// Replace the complete in-memory source set for this evaluation. Omitted
    /// files fall back to disk, including files whose buffers were closed.
    pub fn prepare_with_sources(
        &mut self,
        entry: impl AsRef<Path>,
        inputs: Dict,
        sources: BTreeMap<PathBuf, String>,
    ) -> Result<()> {
        let main = entry_id(entry.as_ref())?;
        let overrides = sources
            .into_iter()
            .map(|(path, text)| Ok((entry_id(&path)?, Bytes::new(text.into_bytes()))))
            .collect::<Result<_>>()?;
        self.main = main;
        self.overrides = overrides;
        self.now = Local::now();
        self.package_roots.get_mut().unwrap().clear();
        if self.inputs != inputs {
            self.library = LazyHash::new(Library::builder().with_inputs(inputs.clone()).build());
            self.inputs = inputs;
        }
        let slots = self.slots.get_mut().unwrap();
        // Do not accumulate files from every entry ever evaluated. Snapshots
        // independently retain any sources still needed by their callers.
        slots.retain(|_, slot| slot.accessed);
        for slot in slots.values_mut() {
            slot.accessed = false;
        }
        Ok(())
    }

    pub fn path_for(&self, id: FileId) -> FileResult<PathBuf> {
        let Some(spec) = id.package() else {
            return id
                .vpath()
                .resolve(&self.root)
                .ok_or(FileError::AccessDenied);
        };
        let mut roots = self.package_roots.lock().unwrap();
        if !roots.contains_key(spec) {
            let root = self.packages.prepare_package(spec, &mut ProgressSink)?;
            roots.insert(spec.clone(), root);
        }
        id.vpath()
            .resolve(&roots[spec])
            .ok_or(FileError::AccessDenied)
    }

    /// Capture the sources actually observed in this evaluation, without disk IO.
    pub fn snapshot(&self) -> SourceSnapshot {
        let sources = self
            .slots
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, slot)| slot.accessed && !slot.source_stale)
            .filter_map(|(&id, slot)| {
                slot.source
                    .as_ref()?
                    .as_ref()
                    .ok()
                    .map(|source| (id, source.clone()))
            })
            .collect();
        SourceSnapshot {
            root: self.root.clone(),
            package_roots: self.package_roots.lock().unwrap().clone(),
            sources,
        }
    }

    fn with_slot<T>(&self, id: FileId, f: impl FnOnce(&mut FileSlot) -> T) -> T {
        let mut slots = self.slots.lock().unwrap();
        let slot = slots.entry(id).or_default();
        if !slot.accessed {
            let bytes = if let Some(bytes) = self.overrides.get(&id) {
                Ok(bytes.clone())
            } else {
                self.path_for(id).and_then(|path| {
                    fs::read(&path)
                        .map(Bytes::new)
                        .map_err(|error| FileError::from_io(error, &path))
                })
            };
            if slot.bytes.as_ref() != Some(&bytes) {
                slot.source_stale = true;
                slot.bytes = Some(bytes);
            }
            slot.accessed = true;
        }
        f(slot)
    }
}

impl World for ProjectWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.with_slot(id, |slot| slot.source(id))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.with_slot(id, |slot| slot.bytes.as_ref().unwrap().clone())
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index)?.get()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let date = match offset {
            None => self.now.date_naive(),
            Some(hours) => {
                let seconds = i32::try_from(hours.checked_mul(3600)?).ok()?;
                let offset = FixedOffset::east_opt(seconds)?;
                self.now.with_timezone(&offset).date_naive()
            }
        };
        Datetime::from_ymd(
            date.year(),
            date.month().try_into().ok()?,
            date.day().try_into().ok()?,
        )
    }
}

#[derive(Default)]
struct FileSlot {
    // Both source() and file() use these same bytes for the whole evaluation.
    bytes: Option<FileResult<Bytes>>,
    source: Option<FileResult<Source>>,
    source_stale: bool,
    accessed: bool,
}

impl FileSlot {
    fn source(&mut self, id: FileId) -> FileResult<Source> {
        if self.source_stale || self.source.is_none() {
            let previous = self.source.take().and_then(Result::ok);
            let source = self.bytes.as_ref().unwrap().clone().and_then(|bytes| {
                let text = std::str::from_utf8(&bytes)?;
                Ok(if let Some(mut source) = previous {
                    source.replace(text);
                    source
                } else {
                    Source::new(id, text.to_owned())
                })
            });
            self.source = Some(source);
            self.source_stale = false;
        }
        self.source.as_ref().unwrap().clone()
    }
}

/// Source provenance for one evaluation. Never reads from the live world or disk.
///
/// Typst sources are cheap, copy-on-write clones: later edits cannot change this
/// snapshot's text or span resolution. This is a record of the bytes observed
/// while evaluating, not an atomic snapshot of the entire filesystem.
#[derive(Clone)]
pub struct SourceSnapshot {
    root: PathBuf,
    package_roots: HashMap<PackageSpec, PathBuf>,
    sources: HashMap<FileId, Source>,
}

impl SourceSnapshot {
    pub fn source(&self, id: FileId) -> Option<&Source> {
        self.sources.get(&id)
    }

    pub fn path_for(&self, id: FileId) -> FileResult<PathBuf> {
        let root = match id.package() {
            Some(spec) => self
                .package_roots
                .get(spec)
                .ok_or_else(|| FileError::NotFound(id.vpath().as_rootless_path().into()))?,
            None => &self.root,
        };
        id.vpath().resolve(root).ok_or(FileError::AccessDenied)
    }

    pub fn range(&self, span: Span) -> Option<Range<usize>> {
        let source = self.source(span.id()?)?;
        span.range().or_else(|| source.range(span))
    }
}

fn entry_id(entry: &Path) -> Result<FileId> {
    if entry.is_absolute()
        || entry
            .components()
            .any(|part| matches!(part, Component::ParentDir))
        || !entry
            .components()
            .any(|part| matches!(part, Component::Normal(_)))
    {
        bail!("entry must be a non-empty project-relative path without '..'");
    }
    Ok(FileId::new(None, VirtualPath::new(entry)))
}

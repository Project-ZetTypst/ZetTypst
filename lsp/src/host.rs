//! Controlled host effects, committed only by the current LSP evaluation.
use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

#[cfg(test)]
mod tests;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WriteFile {
    version: u32,
    id: String,
    path: String,
    content: String,
}

pub struct Writes(Vec<(PathBuf, String)>);

pub fn relative(path: &str) -> Result<PathBuf> {
    ensure!(
        !path.is_empty() && !path.contains('\\'),
        "expected a project-relative path"
    );
    ensure!(
        path.split('/')
            .all(|part| !part.is_empty() && part != "." && part != ".."),
        "invalid path component"
    );
    let path = PathBuf::from(path);
    ensure!(
        path.components()
            .all(|part| matches!(part, Component::Normal(_))),
        "expected a project-relative path"
    );
    Ok(path)
}

// Reject symlinks even when they currently resolve inside the permitted directory.
fn check_path(root: &Path, path: &Path) -> Result<()> {
    let mut current = root.to_owned();
    for part in path.components() {
        current.push(part);
        match std::fs::symlink_metadata(&current) {
            Ok(meta) => {
                ensure!(
                    !meta.file_type().is_symlink(),
                    "write path contains a symlink"
                );
                ensure!(
                    if current == root.join(path) {
                        meta.is_file()
                    } else {
                        meta.is_dir()
                    },
                    "invalid write path type"
                );
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

impl Writes {
    pub fn prepare(
        output: &Value,
        root: &Path,
        allowed: Option<&Path>,
        open: impl Iterator<Item = PathBuf>,
    ) -> Result<Self> {
        let Some(value) = output.get("host.write-file") else {
            return Ok(Self(vec![]));
        };
        let requests: Vec<WriteFile> = serde_json::from_value(value.clone())?;
        if requests.is_empty() {
            return Ok(Self(vec![]));
        }
        let allowed =
            allowed.context("host.write-file requires initializationOptions.hostWriteRoot")?;
        let open: BTreeSet<_> = open.collect();
        let mut ids = BTreeSet::new();
        let mut paths = BTreeSet::new();
        let mut writes = vec![];
        for request in requests {
            ensure!(request.version == 1, "unsupported host.write-file version");
            ensure!(
                !request.id.is_empty() && ids.insert(request.id),
                "empty or duplicate write task ID"
            );
            let path = relative(&request.path)?;
            ensure!(
                path.starts_with(allowed) && path != allowed,
                "write path is outside hostWriteRoot"
            );
            ensure!(!open.contains(&path), "cannot overwrite an open document");
            ensure!(paths.insert(path.clone()), "duplicate write target");
            check_path(root, &path)?;
            writes.push((path, request.content));
        }
        Ok(Self(writes))
    }

    // Called synchronously on the server event loop after its generation check.
    // Atomicity is per file, not a transaction across all announced files.
    pub fn commit(self, root: &Path) -> Result<()> {
        for (path, content) in self.0 {
            check_path(root, &path)?;
            let target = root.join(&path);
            match std::fs::read(&target) {
                Ok(previous) if previous == content.as_bytes() => continue,
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
            let parent = target.parent().context("missing write parent")?;
            std::fs::create_dir_all(parent)?;
            check_path(root, &path)?;
            let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
            temporary.write_all(content.as_bytes())?;
            temporary.as_file().sync_all()?;
            temporary
                .persist(&target)
                .with_context(|| format!("cannot publish {}", path.display()))?;
        }
        Ok(())
    }
}

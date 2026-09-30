//! Persistent-cache provenance, separate from retained Typst source spans.
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub fn content_hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

// Hash encoding is intentionally tied to the executable fingerprint used by LSP.
pub(crate) fn value_hash(value: &impl std::hash::Hash) -> String {
    struct Hasher(Sha256);
    impl std::hash::Hasher for Hasher {
        fn finish(&self) -> u64 {
            unreachable!("only streaming hash is used")
        }
        fn write(&mut self, bytes: &[u8]) {
            self.0.update(bytes);
        }
    }
    let mut state = Hasher(Sha256::new());
    value.hash(&mut state);
    format!("{:x}", state.0.finalize())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    pub path: PathBuf,
    pub hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageDependency {
    /// Ordered local/cache candidates used by typst-kit, without network access.
    pub candidates: Vec<PathBuf>,
    pub resolved: PathBuf,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dependencies {
    pub files: Vec<Dependency>,
    pub packages: Vec<PackageDependency>,
    /// Hash of observed FontBook metadata. Checked against a fresh World on restore.
    pub font_book: Option<String>,
    /// Reasons for refusing persistence, never a license to omit a dependency.
    pub unsupported: Option<String>,
}

impl Dependencies {
    /// Content checks include overlays AND disk: only disk-equivalent snapshots
    /// may cross sessions. No mtimes, no negative-read assumptions, no downloads.
    pub fn matches(&self, root: &Path, sources: &BTreeMap<PathBuf, String>) -> bool {
        if self.unsupported.is_some() || self.files.is_empty() {
            return false;
        }
        if !self.packages.iter().all(|package| {
            package.candidates.iter().find(|path| path.exists()) == Some(&package.resolved)
        }) {
            return false;
        }
        self.files.iter().all(|dependency| {
            let overlay_matches = dependency
                .path
                .strip_prefix(root)
                .ok()
                .and_then(|path| sources.get(path))
                .is_none_or(|text| content_hash(text.as_bytes()) == dependency.hash);
            overlay_matches
                && fs::read(&dependency.path)
                    .is_ok_and(|bytes| content_hash(&bytes) == dependency.hash)
        })
    }

    pub fn contains_text(&self, path: &Path, text: &str) -> bool {
        self.files
            .iter()
            .any(|file| file.path == path && file.hash == content_hash(text.as_bytes()))
    }
}

//! Module for traversing the filesystem and building an in-memory tree hierarchy.

use std::{
    ffi::OsStr,
    fmt,
    fs::Metadata,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
    sync::Arc,
};

pub type FsObjects = Vec<Arc<FsObject>>;

/// Helper struct for formatting file sizes into human-readable strings without heap allocations.
pub struct SizeFormatter(pub u64);

impl fmt::Display for SizeFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size_kb = self.0 as f64 / 1024.0;
        if size_kb < 0.1 {
            write!(f, "{} B", self.0)
        } else {
            write!(f, "{:.1} KiB", size_kb)
        }
    }
}

/// Represents a filesystem entity (file, directory, or symlink) within the tree structure.
pub struct FsObject {
    /// Path to the filesystem object.
    pub path: PathBuf,

    /// Object metadata obtained from the filesystem.
    pub metadata: Metadata,

    /// Directory contents if this entity is a non-empty directory.
    pub content: Option<FsObjects>,
}

impl FsObject {
    /// Creates a new [`FsObject`] instance.
    pub fn new(path: PathBuf, metadata: Metadata, content: Option<FsObjects>) -> Self {
        Self {
            path,
            metadata,
            content,
        }
    }

    pub fn is_file(&self) -> bool {
        self.metadata.is_file()
    }

    pub fn is_dir(&self) -> bool {
        self.metadata.is_dir()
    }

    pub fn is_symlink(&self) -> bool {
        self.metadata.is_symlink()
    }

    pub fn name(&self) -> &str {
        self.path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("unnamed")
    }

    /// Returns a displayable formatter for the file size.
    pub fn size_display(&self) -> SizeFormatter {
        SizeFormatter(self.metadata.len())
    }

    /// Returns a depth-first iterator over this node and all nested [`FsObject`] nodes.
    pub fn recursive_iter(&self) -> impl Iterator<Item = &FsObject> {
        let mut stack = vec![self];
        std::iter::from_fn(move || {
            let current = stack.pop()?;
            if let Some(ref content) = current.content {
                stack.extend(content.iter().map(Arc::as_ref));
            }
            Some(current)
        })
    }

    /// Computes and returns a 64-bit hash of the object based on its path.
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl PartialEq for FsObject {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for FsObject {}

impl Hash for FsObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

/// Holds aggregated counters for a collection of filesystem elements.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct FsSummary {
    pub total_elements: usize,
    pub total_files: usize,
    pub total_directories: usize,
    pub total_symlinks: usize,
}

impl FsSummary {
    /// Computes summary statistics for a slice of [`FsObject`] root nodes.
    pub fn from_objects(fs_objects: &[Arc<FsObject>]) -> Self {
        let mut summary = FsSummary::default();

        for fs_obj in fs_objects {
            for item in fs_obj.recursive_iter() {
                summary.total_elements += 1;
                if item.is_symlink() {
                    summary.total_symlinks += 1;
                } else if item.is_file() {
                    summary.total_files += 1;
                } else if item.is_dir() {
                    summary.total_directories += 1;
                }
            }
        }

        summary
    }
}

impl fmt::Display for FsSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\nObtained:\t{} elements, where:", self.total_elements)?;
        writeln!(f, "\t\t{} files", self.total_files)?;
        writeln!(f, "\t\t{} directories", self.total_directories)?;
        writeln!(f, "\t\t{} symbolic links\n", self.total_symlinks)
    }
}

//! This module created to scan filesystem and store files in tree-style

use std::{
    ffi::OsStr,
    fs::Metadata,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
    sync::Arc,
};

use std::string::String;

pub type FsObjects = Vec<Arc<FsObject>>;

/// A file system element for building a directory tree in RAM and accessing metadata.
pub struct FsObject {
    /// Path to object
    pub path: PathBuf,

    /// Object metadata
    pub metadata: Metadata,

    /// Contain contents of the folder.
    /// If this file or folder is empty - the value is None
    pub content: Option<FsObjects>,
}

impl FsObject {
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

    pub fn size_string(&self) -> String {
        format!("{} kB", self.metadata.len() / 1000)
    }

    /// Return iterator over each FSObject
    pub fn recursive_iter(&self) -> impl Iterator<Item = &FsObject> {
        let mut stack = vec![self];
        std::iter::from_fn(move || {
            if let Some(current) = stack.pop() {
                if let Some(ref content) = current.content {
                    stack.extend(content.iter().map(Arc::as_ref));
                }
                Some(current)
            } else {
                None
            }
        })
    }

    /// Return Hash of FSObject that are obtained with DefaultHasher
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl Hash for FsObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

pub fn show_fs_objects_summary(fs_objects: &FsObjects) {
    let mut total_elements = 0;
    let mut total_files = 0;
    let mut total_directories = 0;
    let mut total_symlinks = 0;

    for fs_obj in fs_objects {
        for item in fs_obj.recursive_iter() {
            total_elements += 1;
            if item.is_file() {
                total_files += 1;
            } else if item.is_dir() {
                total_directories += 1;
            } else if item.is_symlink() {
                total_symlinks += 1;
            }
        }
    }

    println!("\nObtained:\t{} elements, where:", total_elements);
    println!("\t\t{} files", total_files);
    println!("\t\t{} directories", total_directories);
    println!("\t\t{} symbolic links\n", total_symlinks);
}

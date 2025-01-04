//! This module created to scan filesystem and store files in tree-style.

use std::{
    ffi::OsStr,
    fs::Metadata,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
    sync::Arc,
};

#[cfg(target_family = "unix")]
use std::os::unix::fs::MetadataExt;

#[cfg(target_family = "windows")]
use std::os::windows::prelude::*;

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
            .expect(&format!("Failed to retrieve file name: {:?}", &self.path))
    }

    pub fn size(&self) -> String {
        let size_string;

        #[cfg(target_family = "unix")]
        {
            size_string = format!("{} kB", self.metadata.size() / 1000);
        }

        #[cfg(target_family = "windows")]
        {
            size_string = format!("{} kB", self.metadata.file_size() / 1000);
        }

        size_string
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

    /// Return iterator over each FSObject that is a file
    pub fn file_iter(&self) -> impl Iterator<Item = &FsObject> {
        let mut stack = vec![self];
        std::iter::from_fn(move || {
            while let Some(current) = stack.pop() {
                if current.is_file() {
                    return Some(current); // Return the current object if it is a file
                } else {
                    if let Some(ref content) = current.content {
                        stack.extend(content.iter().map(Arc::as_ref));
                    }
                }
            }
            None
        })
    }

    /// Return iterator over each FSObject that is a directory
    pub fn dir_iter(&self) -> impl Iterator<Item = &FsObject> {
        let mut stack = vec![self];
        std::iter::from_fn(move || {
            while let Some(current) = stack.pop() {
                if current.is_dir() {
                    // Return the current directory object
                    if let Some(ref content) = current.content {
                        // Push content of directories to stack
                        stack.extend(content.iter().map(Arc::as_ref));
                    }
                    return Some(current);
                } else if let Some(ref content) = current.content {
                    // Push content of directories to stack
                    stack.extend(content.iter().map(Arc::as_ref));
                }
            }
            None
        })
    }

    /// Return iterator over each FSObject that is a symbolic link. Not ready yet!
    pub fn symlink_iter(&self) -> impl Iterator<Item = &FsObject> {
        let mut stack = vec![self];
        std::iter::from_fn(move || {
            while let Some(current) = stack.pop() {
                if current.is_symlink() {
                    return Some(current); // Return the current object if it is a file
                } else {
                    if let Some(ref content) = current.content {
                        stack.extend(content.iter().map(Arc::as_ref));
                    }
                }
            }
            None
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
        self.content.hash(state);
    }
}

pub fn show_fs_objects_summary(fs_objects: &FsObjects) {
    let total_elements: usize = fs_objects
        .iter()
        .map(|fs_obj| fs_obj.recursive_iter().count())
        .sum();

    let total_files: usize = fs_objects
        .iter()
        .map(|fs_obj| fs_obj.file_iter().count())
        .sum();

    let total_directories: usize = fs_objects
        .iter()
        .map(|fs_obj| fs_obj.dir_iter().count())
        .sum();

    let total_symlinks: usize = fs_objects
        .iter()
        .map(|fs_obj| fs_obj.symlink_iter().count())
        .sum();

    println!("\nObtained:\t{} elements, where:", total_elements);
    println!("\t\t{} files", total_files);
    println!("\t\t{} directories", total_directories);
    println!("\t\t{} symbolic links\n", total_symlinks);
}

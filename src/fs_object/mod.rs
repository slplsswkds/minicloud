//! This module created to scan filesystem and store files in tree-style.

use std::{
    fs,
    fs::Metadata,
    ffi::OsStr,
    io::Error,
    path::PathBuf,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

#[cfg(target_family = "unix")]
use std::os::unix::fs::MetadataExt;

#[cfg(target_family = "windows")]
use std::os::windows::prelude::*;

use std::string::String;

/// A file system element for building a directory tree in RAM and accessing metadata.
#[derive(Debug)]
pub struct FSObject {
    /// Path to object
    pub path: PathBuf,

    /// Object metadata
    metadata: Metadata,

    /// Contain contents of the folder.
    /// If this file or folder is empty - the value is None
    pub content: Option<Vec<Arc<FSObject>>>,
}

impl FSObject {
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
            .expect("Failed to retrieve file name")
    }

    pub fn size(&self) -> String {
        let size_string;

        #[cfg(target_family = "unix")]
        {
            size_string = format!("{} kB", self.metadata.size() / 1000);
        }

        #[cfg(target_family = "windows")]
        return self.metadata.file_size();

        size_string
    }

    /// Return iterator over each FSObject
    pub fn recursive_iter(&self) -> impl Iterator<Item=&FSObject> {
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
    pub fn file_iter(&self) -> impl Iterator<Item=&FSObject> {
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
    pub fn dir_iter(&self) -> impl Iterator<Item=&FSObject> {
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
    pub fn symlink_iter(&self) -> impl Iterator<Item=&FSObject> {
        // !!!not implemented yet.
        std::iter::empty::<&FSObject>()
    }

    /// Return Hash of FSObject that are obtained with DefaultHasher
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl Hash for FSObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
        self.content.hash(state);
    }
}

/// Scan vector of PathBuf recursively into FSObject
///
/// This function will return an error in the following situations, but is not limited to just these cases:
/// - The user lacks permissions to perform metadata call on path.
/// - path does not exist.

pub fn content_recursively(paths: &Vec<PathBuf>) -> Result<Vec<Arc<FSObject>>, Error> {
    let mut fs_objects_root: Vec<Arc<FSObject>> = Vec::new();

    for path in paths {
        let metadata;

        if path.is_symlink() {
            metadata = match path.symlink_metadata() {
                Ok(metadata) => metadata,
                Err(err) => {
                    eprintln!("{:?}: {err}", path);
                    continue; // skip file !!!!!!!!!!!!!!!!!!!!!! fix this in the future
                }
            };
        } else {
            metadata = match path.metadata() {
                Ok(metadata) => metadata,
                Err(err) => {
                    eprintln!("{:?}: {err}", path);
                    let empty_vec: Vec<Arc<FSObject>> = Vec::new();
                    return Ok(empty_vec);                       // not good...
                }
            };
        }

        let mut fs_object = FSObject {
            path: path.clone(),
            metadata,
            content: None,
        };

        if path.is_dir() {
            match read_dir_content(&path) {
                Ok(dir_content) => {
                    if dir_content.len() > 0 {
                        fs_object.content = Some(content_recursively(&dir_content)?);
                    }
                }
                Err(err) => {
                    eprintln!("{:?}: {err}", path);
                    let empty_vec: Vec<Arc<FSObject>> = Vec::new();
                    return Ok(empty_vec);
                }
            }
        }

        fs_objects_root.push(Arc::new(fs_object));
    }
    Ok(fs_objects_root)
}

/// Returns a list of files, directories, and symbolic links in a directory
///
/// This function will return an error in the following situations, but is not limited to just these cases:
/// - The provided path doesn't exist.
/// - The process lacks permissions to view the contents.
/// - The path points at a non-directory file.
fn read_dir_content(path: &PathBuf) -> Result<Vec<PathBuf>, Error> {
    let mut paths: Vec<PathBuf> = Vec::new();

    for entry_result in fs::read_dir(path)? { // !!! handle this !!!. error beginning is there
        let entry = entry_result.expect("MY_ERROR_3");
        paths.push(entry.path());
    }

    Ok(paths)
}

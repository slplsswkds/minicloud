//! This module created to scan filesystem and store files in tree-style.

use std::{
    fs,
    fs::Metadata,
    ffi::OsStr,
    io::Error,
    path::PathBuf,
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
    path: PathBuf,

    /// Object metadata
    metadata: Metadata,

    /// Containt contents of the folder.
    /// If this file or folder is empty - the value is None
    pub content: Option<Vec<FSObject>>,
}

impl FSObject {
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
}

/// Scan vector of PathBuf recursively into FSObject
///
/// This function will return an error in the following situations, but is not limited to just these cases:
/// - The user lacks permissions to perform metadata call on path.
/// - path does not exist.

pub fn content_recursively(paths: &Vec<PathBuf>) -> Result<Vec<FSObject>, Error> {
    let mut fs_objects_root: Vec<FSObject> = Vec::new();

    for path in paths {
        let metadata;

        if path.is_symlink() {
            metadata = match path.symlink_metadata() {
                Ok(metadata) => metadata,
                Err(err) => {
                    eprintln!("{:?}: {err}", path);
                    // let empty_vec: Vec<FSObject> = Vec::new();
                    // return Ok(empty_vec);                       // not good...
                    continue; // skip file !!!!!!!!!!!!!!!!!!!!!! fix this in the future
                }
            };
        } else {
            metadata = match path.metadata() {
                Ok(metadata) => metadata,
                Err(err) => {
                    eprintln!("{:?}: {err}", path);
                    let empty_vec: Vec<FSObject> = Vec::new();
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
            fs_object.content = Some(Vec::new());
            match read_dir_content(&path) {
                Ok(dir_content) => {
                    if dir_content.len() > 0 {
                        fs_object.content = Some(content_recursively(&dir_content).expect("MY_ERROR_1"))
                    }
                }
                Err(err) => {
                    eprintln!("{:?}: {err}", path);
                    let empty_vec: Vec<FSObject> = Vec::new();
                    return Ok(empty_vec);
                }
            }
        }

        fs_objects_root.push(fs_object);
    }
    Ok(fs_objects_root)
}

/// Returns a list of files, directories, and symbolic links in a directory
///
/// This function will return an error in the following situations, but is not limited to just these cases:
/// - The provided path doesn’t exist.
/// - The process lacks permissions to view the contents.
/// - The path points at a non-directory file.
fn read_dir_content(path: &PathBuf) -> Result<Vec<PathBuf>, Error> {
    let mut paths: Vec<PathBuf> = Vec::new();

    for entry_result in fs::read_dir(path)? { // !!! handle this !!!. error begining is there
        let entry = entry_result.expect("MY_ERROR_3");
        paths.push(entry.path());
    }

    Ok(paths)
}

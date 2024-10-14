use std::{
    fs,
    io::Error,
    path::{PathBuf, Path},
    sync::Arc,
};
use crate::fs_object::{FSObject, FsObjects};

/// Scan vector of PathBuf recursively into FSObject
///
/// This function will return an error in the following situations, but is not limited to just these cases:
/// - The user lacks permissions to perform metadata call on path.
/// - path does not exist.
pub fn content_recursively(paths: &[PathBuf]) -> Result<FsObjects, Error> {
    let mut fs_objects_root: FsObjects = Vec::new();

    for path in paths {
        let metadata = match get_metadata(path) {
            Ok(metadata) => metadata,
            Err(err) => {
                eprintln!("{:?}: {err}", path);
                continue; // skip path
            }
        };

        let mut fs_object = FSObject {
            path: path.clone(),
            metadata,
            content: None,
        };

        if path.is_dir() {
            match read_dir_content(path) {
                Ok(dir_content) => {
                    if !dir_content.is_empty() {
                        fs_object.content = Some(content_recursively(&dir_content)?);
                    }
                }
                Err(err) => {
                    eprintln!("{:?}: {err}", path);
                    continue; // skip directory with errors
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
fn read_dir_content(path: &Path) -> Result<Vec<PathBuf>, Error> {
    fs::read_dir(path)?
        .filter_map(|entry| {
            match entry {
                Ok(dir_entry) => Some(Ok(dir_entry.path())),
                Err(err) => {
                    eprintln!("Error reading directory entry: {err}. Skipping...");
                    None
                }
            }
        })
        .collect()
}

/// Gets metadata for a given path (handling symbolic links)
fn get_metadata(path: &Path) -> Result<fs::Metadata, Error> {
    if path.is_symlink() {
        path.symlink_metadata()
    } else {
        path.metadata()
    }
}

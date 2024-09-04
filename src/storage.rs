use std::{
    fs,
    io::Error,
    path::PathBuf,
    sync::Arc,
};
use crate::fs_object::FSObject;

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
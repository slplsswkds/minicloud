use crate::fs_object::{FsObject, FsObjects};
use std::{
    fs,
    io::Result,
    path::{Path, PathBuf},
    sync::Arc,
};

/// Recursively scans a vector of PathBuf and constructs a vector of FSObject.
///
/// This function traverses the provided paths and creates FSObject instances for each valid path.
/// It skips paths where an error occurs while retrieving metadata or reading directory contents.
/// Possible error conditions include, but are not limited to:
/// - Insufficient permissions to access the specified path.
/// - The specified path does not exist.
///
/// The function will return a Result containing a vector of FSObjects on success,
/// or an error if any issues arise during the scanning process.
pub fn content_recursively(paths: &[PathBuf]) -> Result<FsObjects> {
    let mut fs_objects_root: FsObjects = Vec::new();

    for path in paths.iter() {
        match process_single_path(path) {
            Ok(fs_object) => fs_objects_root.push(Arc::new(fs_object)),
            Err(err) => tracing::warn!("{err}: {:?}", path),
        }
    }
    Ok(fs_objects_root)
}

/// Processes a single PathBuf to retrieve its metadata and, if it's a directory,
/// recursively scans its contents into an [`FsObject`].
///
/// This function attempts to obtain the metadata for the specified path and create
/// an FSObject instance. If the path is a directory, it reads the contents and
/// calls itself recursively to populate the FSObject's content field.
///
/// Possible error conditions include:
/// - The path does not exist or is inaccessible.
/// - An error occurs while reading the directory contents.
///
/// The function returns a Result containing the constructed FSObject on success,
/// or an error if the path processing fails.
fn process_single_path(path: &Path) -> Result<FsObject> {
    let metadata = get_metadata(path)?;
    let mut fs_object = FsObject::new(path.to_path_buf(), metadata, None);

    if path.is_dir() && !path.is_symlink() {
        let dir_content = read_dir_content(path)?;
        if !dir_content.is_empty() {
            fs_object.content = Some(content_recursively(&dir_content)?);
        }
    }

    Ok(fs_object)
}

/// Returns a list of files, directories, and symbolic links in a directory
///
/// This function will return an error in the following situations, but is not limited to just these cases:
/// - The provided path doesn't exist.
/// - The process lacks permissions to view the contents.
/// - The path points at a non-directory file.
fn read_dir_content(path: &Path) -> Result<Vec<PathBuf>> {
    fs::read_dir(path)?
        .filter_map(|entry| match entry {
            Ok(dir_entry) => Some(Ok(dir_entry.path())),
            Err(err) => {
                tracing::warn!("Failed to read directory entry: {err}. Skipping...");
                None
            }
        })
        .collect()
}

#[inline]
fn get_metadata(path: &Path) -> Result<fs::Metadata> {
    path.metadata().or_else(|err| {
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                // Maybe it's symbolic link...
                path.symlink_metadata()
            }
            _ => Err(err),
        }
    })
}

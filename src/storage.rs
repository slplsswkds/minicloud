use std::{
    fs,
    io::Result,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::fs_object::{FsObject, FsObjects};

/// Recursively scans a slice of paths and constructs a hierarchy of [`FsObject`] instances.
///
/// Any errors encountered while processing individual paths (e.g. permission issues)
/// are logged as warnings without halting the scanning process.
pub fn content_recursively(paths: &[PathBuf]) -> Result<FsObjects> {
    let mut fs_objects_root = Vec::with_capacity(paths.len());

    for path in paths {
        match process_single_path(path.clone()) {
            Ok(fs_object) => fs_objects_root.push(Arc::new(fs_object)),
            Err(err) => tracing::warn!("{err}: {:?}", path),
        }
    }
    Ok(fs_objects_root)
}

/// Processes a single [`PathBuf`] to construct its [`FsObject`] representation.
///
/// Takes ownership of `path` to avoid redundant memory allocations.
fn process_single_path(path: PathBuf) -> Result<FsObject> {
    let metadata = get_metadata(&path)?;

    let is_dir = metadata.is_dir();
    let is_symlink = metadata.is_symlink();

    let mut fs_object = FsObject::new(path, metadata, None);

    if is_dir && !is_symlink {
        let dir_content = read_dir_content(&fs_object.path)?;
        if !dir_content.is_empty() {
            fs_object.content = Some(content_recursively(&dir_content)?);
        }
    }

    Ok(fs_object)
}

/// Reads the contents of a directory and returns a vector of [`PathBuf`] entries.
fn read_dir_content(path: &Path) -> Result<Vec<PathBuf>> {
    let entries = fs::read_dir(path)?
        .filter_map(|entry| match entry {
            Ok(dir_entry) => Some(dir_entry.path()),
            Err(err) => {
                tracing::warn!("Failed to read directory entry: {err}. Skipping...");
                None
            }
        })
        .collect();

    Ok(entries)
}

/// Retrieves metadata for a given path without following symbolic links.
#[inline]
fn get_metadata(path: &Path) -> Result<fs::Metadata> {
    path.symlink_metadata()
}

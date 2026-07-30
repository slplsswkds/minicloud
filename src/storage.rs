use std::{
    fs::{read_dir, DirEntry},
    io::Result,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::fs_object::{FsObject, FsObjects};

/// Recursively scans a slice of paths sequentially in a single thread.
///
/// Optimized to minimize system calls (`stat`/`lstat`) and memory allocations.
pub fn content_recursively(paths: &[PathBuf]) -> Result<FsObjects> {
    let mut fs_objects_root = Vec::with_capacity(paths.len());

    for path in paths {
        match process_root_path(path.clone()) {
            Ok(fs_object) => fs_objects_root.push(Arc::new(fs_object)),
            Err(err) => tracing::warn!("{err}: {:?}", path),
        }
    }

    Ok(fs_objects_root)
}

/// Processes a root path (which comes directly as a `PathBuf`, not from `read_dir`).
fn process_root_path(path: PathBuf) -> Result<FsObject> {
    let metadata = path.symlink_metadata()?;
    let is_dir = metadata.is_dir();
    let is_symlink = metadata.is_symlink();

    let content = if is_dir && !is_symlink {
        scan_dir_content(&path)
    } else {
        None
    };

    Ok(FsObject::new(path, metadata, content))
}

/// Reads a directory sequentially and constructs child [`FsObject`]s directly
/// without intermediate `Vec<PathBuf>` allocations.
fn scan_dir_content(path: &Path) -> Option<FsObjects> {
    let read_dir = match read_dir(path) {
        Ok(rd) => rd,
        Err(err) => {
            tracing::warn!("Failed to read directory {:?}: {err}", path);
            return None;
        }
    };

    let mut children = Vec::new();

    for entry_result in read_dir {
        let entry = match entry_result {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!("Failed to read directory entry in {:?}: {err}", path);
                continue;
            }
        };

        match process_dir_entry(entry) {
            Ok(fs_object) => children.push(Arc::new(fs_object)),
            Err(err) => tracing::warn!("Failed to process entry in {:?}: {err}", path),
        }
    }

    if children.is_empty() {
        None
    } else {
        Some(children)
    }
}

fn process_dir_entry(entry: DirEntry) -> Result<FsObject> {
    let metadata = entry.metadata()?;
    let is_dir = metadata.is_dir();
    let is_symlink = metadata.is_symlink();

    let path = entry.path();

    let content = if is_dir && !is_symlink {
        scan_dir_content(&path)
    } else {
        None
    };

    Ok(FsObject::new(path, metadata, content))
}

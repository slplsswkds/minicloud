use std::ffi::OsStr;
use std::fs;
use std::fs::Metadata;
use std::io::Error;
use std::path::PathBuf;

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

    pub fn name(&self) -> &str {
        self.path
            .file_name()
            .and_then(OsStr::to_str)
            .expect("Failed to retrieve file name")
    }
}

pub fn content_recursively(paths: &Vec<PathBuf>) -> Result<Vec<FSObject>, Error> {
    let mut fs_objects_root: Vec<FSObject> = Vec::new();

    for path in paths {
        let mut fs_object = FSObject {
            path: path.clone(),
            metadata: path.metadata()?,
            content: None,
        };

        if path.is_dir() {
            fs_object.content = Some(Vec::new());
            let dir_content = read_dir_content(&path)?;
            fs_object.content = Some(content_recursively(&dir_content).unwrap())
            //fs_object.content = Some(dir_content);
        }

        fs_objects_root.push(fs_object);
    }
    Ok(fs_objects_root)
}

/// Returns a list of files, directories, and symbolic links in a directory
fn read_dir_content(path: &PathBuf) -> Result<Vec<PathBuf>, Error> {
    let mut paths: Vec<PathBuf> = Vec::new();

    for entrie in fs::read_dir(path)? {
        let entrie = entrie?;
        paths.push(entrie.path());
    }

    Ok(paths)
}

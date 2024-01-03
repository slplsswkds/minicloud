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
        let metadata = match path.metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                eprintln!("{:?}: {err}", path);
                let empty_vec: Vec<FSObject> = Vec::new();
                return Ok(empty_vec); 
            }
        };
        let mut fs_object = FSObject {
            path: path.clone(),
            metadata: metadata,
            content: None,
        };

        if path.is_dir() {
            fs_object.content = Some(Vec::new());
            match read_dir_content(&path) {
                Ok(dir_content) => {
                    if dir_content.len() > 0 {
                        fs_object.content = Some(content_recursively(&dir_content).expect("MY_ERROR_1"))
                    }
                },
                Err(err) => {
                    eprintln!("{:?}: {err}", path);
                    let empty_vec: Vec<FSObject> = Vec::new();
                    return Ok(empty_vec);
                }
            }
            //fs_object.content = Some(content_recursively(&dir_content).unwrap())
            //fs_object.content = Some(dir_content);
        }

        fs_objects_root.push(fs_object);
    }
    Ok(fs_objects_root)
}

/// Returns a list of files, directories, and symbolic links in a directory
fn read_dir_content(path: &PathBuf) -> Result<Vec<PathBuf>, Error> {
    let mut paths: Vec<PathBuf> = Vec::new();

    for entrie in fs::read_dir(path)? { // !!! handle this !!!. error begining is there
        let entrie = entrie.expect("MY_ERROR_3");
        paths.push(entrie.path());
    }

    Ok(paths)
}

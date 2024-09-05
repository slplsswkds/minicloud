use clap::Parser;
use std::path::PathBuf;

/// A program for transferring files between devices via HTTP with an HTML interface
// Store parsed values from CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Set directories and files that will be distributed
    pub paths: Vec<PathBuf>,

    /// Port number
    #[arg(short = 'p', long, default_value_t = 42666)]
    pub port: u16,

    /// Use the absolute path to the files
    #[arg(short = 'c', long, default_value_t = false)]
    pub canonical: bool,
}

impl Args {
    /// Prepare vector of paths for using. Remove wrong paths and print errors.
    pub fn prepare_data(&mut self) -> Result<(), ()> {
        if self.canonical {
            self.canonicalize_paths();
        } else {
            self.remove_inaccessible_paths()
                .iter()
                .for_each(|wrong_path| {
                    eprintln!("Error: Path {:?} does not exist", wrong_path);
                });
        }

        self.remove_repeated_paths();

        Ok(())
    }

    /// Removes repeated elements in the vector
    fn remove_repeated_paths(&mut self) {
        self.paths.sort();
        self.paths.dedup();
    }

    /// Returns a vector of paths that do not exist in the file system or cannot be accessed.
    ///
    /// This function will traverse the symbolic links to get information about the target file.
    /// If you cannot access the file's metadata, e.g. g. due to a permission error or faulty symlinks,
    /// this will copy the path into a vector of wrong paths
    fn remove_inaccessible_paths(&mut self) -> Vec<PathBuf> {
        let mut wrong_paths = Vec::new();

        self
            .paths
            .retain(|path| {
                if path.exists() {
                    true
                } else {
                    wrong_paths.push(path.clone());
                    false
                }
            });

        wrong_paths
    }

    /// Canonicalize all paths
    fn canonicalize_paths(&mut self) {
        for path in self.paths.iter_mut() {
            match path.canonicalize() {
                Ok(canonical_path) => *path = canonical_path,
                Err(e) => eprintln!("Warning: Could not canonicalize path {:?}: {}", path, e),
            }
        }
    }
}

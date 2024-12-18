use clap::Parser;
use std::path::PathBuf;

/// A program for transferring files between devices via HTTP with an HTML interface
// Store parsed values from CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Set directories and files that will be distributed
    #[arg(required = false)]
    pub paths: Vec<PathBuf>,

    /// Port number
    #[arg(short = 'p', long, default_value_t = 42666)]
    pub port: u16,
}

impl Args {
    /// Prepare vector of paths for using. Remove wrong paths and print errors.
    pub fn prepare_paths(&mut self) -> Result<(), ()> {
        let wrong_paths = self.canonicalize_paths();

        wrong_paths
            .map_err(|err| {
                err.iter().for_each(|(path, err)| {
                    eprintln!("Error: {:?} {:?}", path, err.to_string());
                })
            })?;

        self.remove_repeated_paths();
        Ok(())
    }

    /// Removes repeated elements in the vector
    fn remove_repeated_paths(&mut self) {
        self.paths.sort();
        self.paths.dedup();
    }

    /// Canonicalize all paths
    ///
    /// Returns (paths, error) that cannot be canonicalized
    fn canonicalize_paths(&mut self) -> Result<(), Vec<(PathBuf, std::io::Error)>> {
        let wrong_paths: Vec<(PathBuf, std::io::Error)> = self
            .paths
            .iter()
            .filter_map(|path| {
                path.canonicalize().err().map(|err| (path.clone(), err))
            })
            .collect();

        match wrong_paths.is_empty() {
            true => Ok(()),
            false => Err(wrong_paths)
        }
    }
}

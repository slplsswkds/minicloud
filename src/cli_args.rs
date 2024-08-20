//! Еhis file contains elements for storing and preparing data entered as command line arguments

use clap::Parser;
use std::path::PathBuf;

/// Store parsed values from CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Set directories and files that will be distributed
    #[arg(short = 'p', long, value_delimiter = ',')]
    pub paths: Vec<PathBuf>,

    /// Display the absolute path to the files
    #[arg(short = 'c', long, default_value_t = false)]
    pub canonical: bool,

    /// Turn of generating html file for WEB view
    #[arg(short = 'n', long, default_value_t = false)]
    pub no_html: bool,
}

impl Args {
    /// Prepare vector of paths for using. Remove wrong paths and print errors.
    pub fn prepare_data(&mut self) -> Result<(), ()> {
        self.optimize_paths();

        let validation = self.validate_paths();

        if let Err(errors) = validation {
            errors
                .iter()
                .for_each(|wrong_path| println!("Error: Path {:?} is not exist", wrong_path));
            return Err(()); // Close minicloud
        }

        if self.canonical {
            self.canonicalize_paths();
        }

        self.paths.iter().for_each(|path| println!("{path:?}")); // print paths from args
        Ok(())
    }

    /// Removes consecutive repeated elements in the vector
    fn optimize_paths(&mut self) {
        self.paths.dedup();
    }

    /// Returns vector of paths that do not exist in the file system
    fn validate_paths(&self) -> Result<(), Vec<PathBuf>> {
        let errors: Vec<PathBuf> = self
            .paths
            .iter()
            .filter(|path| !path.exists())
            .cloned()
            .collect();

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(())
    }

    /// canonicalize all pathes
    fn canonicalize_paths(&mut self) {
        for path in self.paths.iter_mut() {
            *path = path.canonicalize().unwrap();
        }
    }
}

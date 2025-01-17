use std::path;
use clap::ArgGroup;
use clap::Parser;
use std::path::PathBuf;
use tracing::{error, warn};

/// A program for transferring files between devices via HTTP with an HTML interface
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(&["receive", "paths"]),
))]
pub struct Args {
    /// Set directories and files that will be distributed (only in transmitter mode)
    #[arg(required = false, value_name = "FILE_OR_DIR", conflicts_with = "receive")]
    pub paths: Vec<PathBuf>,

    /// Port number
    #[arg(short = 'p', long, default_value_t = 48666)]
    pub port: u16,

    /// The application mode in which clients upload files to the server
    #[arg(long, short = 'r', default_value_t = false, requires = "received_files_path")]
    pub receive: bool,

    /// The path where to save the received files (only in receiver mode)
    #[arg(long, default_value = "/tmp/minicloud", value_name = "DIR", requires = "receive")]
    pub received_files_path: PathBuf,

    /// Maximum size of received files in MiB
    #[arg(long, short = 's', default_value_t = 50)]
    pub max_received_file_size: usize,
}
impl Args {
    pub fn prepare_paths(&mut self) {
        for i in (0..self.paths.len()).rev() {  // Iterate from the end
            let path = &mut self.paths[i];

            match path.canonicalize() {
                Ok(canonicalized) => {
                    *path = canonicalized;
                }
                Err(err) => {
                    warn!("Failed to canonicalize path {:?}: {}. Skipping...", path, err);
                    self.paths.remove(i);  // Remove the element if canonicalization failed
                }
            }
        }

        self.remove_repeated_paths();
    }

    fn remove_repeated_paths(&mut self) {
        self.paths.sort();
        self.paths.dedup();
    }

}

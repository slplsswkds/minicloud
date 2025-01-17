use clap::ArgGroup;
use clap::Parser;
use std::path::PathBuf;
use tracing::error;

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
    #[arg(
        required = false,
        value_name = "FILE_OR_DIR",
        conflicts_with = "receive"
    )]
    pub paths: Vec<PathBuf>,

    /// Port number
    #[arg(short = 'p', long, default_value_t = 48666, require_equals = true)]
    pub port: u16,

    /// The application mode in which clients upload files to the server
    #[arg(
        long,
        short = 'r',
        default_value_t = false,
        requires = "received_files_path"
    )]
    pub receive: bool,

    /// The path where to save the received files (only in receiver mode)
    #[arg(
        long,
        value_name = "DIR",
        requires = "receive",
        require_equals = true
    )]
    pub received_files_path: PathBuf,

    /// Maximum size of received files in MiB
    #[arg(long, short = 's', default_value_t = 10, require_equals = true)]
    pub max_received_file_size: usize,
}
impl Args {
    /// Prepare vector of paths for using. Remove wrong paths and print errors.
    pub fn prepare_paths(&mut self) -> Result<(), ()> {
        let wrong_paths = self.canonicalize_paths();

        wrong_paths.map_err(|err| {
            err.iter().for_each(|(path, err)| {
                error!("{:?} {:?}", path, err.to_string());
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
            .filter_map(|path| path.canonicalize().err().map(|err| (path.clone(), err)))
            .collect();

        match wrong_paths.is_empty() {
            true => Ok(()),
            false => Err(wrong_paths),
        }
    }
}

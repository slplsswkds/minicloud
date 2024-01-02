use colored::Colorize;
use std::{fs::canonicalize, path::PathBuf};

pub fn parse_paths(str_path: &Vec<String>) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for arg in str_path {
        let path = PathBuf::from(&arg);
        if path.exists() {
            let full_path = canonicalize(&path).unwrap();
            files.push(full_path);
        } else if path.is_dir() {
            println!(
                "\"{}\" is directory. This is not yet supported. Ignoring the argument...",
                arg.yellow()
            );
        } else {
            println!(
                "File \"{}\" not found. Ignoring the argument...",
                arg.yellow()
            );
        }
    }
    files.dedup();
    return files;
}

pub fn files_show(files: &Vec<PathBuf>) {
    for file in files.into_iter() {
        println!("{}", file.to_str().unwrap().cyan());
    }
}

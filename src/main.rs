mod cli_args;
mod fs_object;
mod html_page;

use std::collections::HashMap;
use axum::{routing::get, Router, extract::State, response::Html};
use clap::Parser;
use fs_object::content_recursively;
use std::sync::Arc;
use crate::fs_object::FSObject;

#[tokio::main]
async fn main() {
    let mut cli_args = cli_args::Args::parse();

    if cli_args.prepare_data().is_err() { return; }

    if cli_args.paths.is_empty() { return; }

    // Get files tree
    let fs_objects = match content_recursively(&cli_args.paths) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            return; // close minicloud
        }
    };

    // Debug info about obtained files, directories, and symbolic links
    #[cfg(debug_assertions)]
    {
        let total_elements: usize = fs_objects
            .iter()
            .map(|fs_obj| fs_obj.recursive_iter().count())
            .sum();

        let total_files: usize = fs_objects
            .iter()
            .map(|fs_obj| fs_obj.file_iter().count())
            .sum();

        let total_directories: usize = fs_objects
            .iter()
            .map(|fs_obj| fs_obj.dir_iter().count())
            .sum();

        let total_symlinks: usize = fs_objects
            .iter()
            .map(|fs_obj| fs_obj.symlink_iter().count())
            .sum();

        println!("\nObtained:\t{} elements, where:", total_elements);
        println!("\t\t{} files", total_files);
        println!("\t\t{} directories", total_directories);
        println!("\t\t{} symbolic links\n", total_symlinks);
    }

    print!("Generating HTML...");
    let (page, hash_map) = html_page::html_page(&fs_objects);
    println!(" OK");

    let mut app = Router::new();

    app = app.route("/", get(root_handler)
        .with_state(Arc::new(Html(page))),
    );

    app = app.route("/dl", get(download_handler)
        .with_state(Arc::new(hash_map)),
    );


    // #[cfg(debug_assertions)]
    // println!("{}", html_page::html_page(&fs_objects));

    //----------------------------------------------
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("\nlistening on 127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn root_handler(page: State<Arc<Html<String>>>) -> Html<String> {
    (**page).clone()
}

async fn download_handler(
    _state: State<Arc<HashMap<u64, Arc<FSObject>>>>,
) -> Html<String> {
    // Your handler logic here
    Html("test string".to_string())
}
mod cli_args;
mod fs_object;
mod html_page;
mod server;

use axum::{routing::get, Router, response::Html};
use clap::Parser;
use fs_object::content_recursively;
use std::sync::Arc;
use crate::server::*;

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


    let hash_map_state = Arc::new(hash_map);

    let mut app = Router::new();

    app = app.route("/", get(root_handler)
        .with_state(Arc::new(Html(page))),
    );

    app = app.route("/dl", get(download_handler)
        .with_state(hash_map_state.clone()),
    );

    app = app.route("/pw", get(preview_handler)
        .with_state(hash_map_state),
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

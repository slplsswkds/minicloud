mod cli_args;
mod fs_object;
mod storage;
mod html_page;
mod server;

use std::net::SocketAddr;
use axum::{routing::get, Router, response::Html};
use clap::Parser;
use storage::content_recursively;
use std::sync::Arc;
use crate::fs_object::show_fs_objects_summary;
use crate::server::*;

#[tokio::main]
async fn main() {
    let mut cli_args = cli_args::Args::parse();

    if cli_args.prepare_paths().is_err() { return; }

    if cli_args.paths.is_empty() { return; }

    // Get files tree
    let fs_objects = match content_recursively(&cli_args.paths) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            return; // close minicloud
        }
    };

    // Info about obtained files, directories, and symbolic links
    show_fs_objects_summary(&fs_objects);

    print!("Generating HTML...");
    let (page, hash_map) = html_page::html_page(&fs_objects);
    println!(" OK");

    let hash_map_state = Arc::new(hash_map);

    let app = Router::new()
        .route("/", get(root_handler).with_state(Arc::new(Html(page))))
        .route("/dl", get(download_handler).with_state(hash_map_state.clone()))
        .route("/pw", get(preview_handler).with_state(hash_map_state));

    //----------------------------------------------

    let local_ip = local_ip_address::local_ip().unwrap();
    let socket_addr = SocketAddr::new(local_ip, cli_args.port);
    let listener = tokio::net::TcpListener::bind(socket_addr);

    println!("\nlistening on {}:{}", socket_addr.ip(), socket_addr.port());

    axum::serve(listener.await.unwrap(), app).await.unwrap();
}

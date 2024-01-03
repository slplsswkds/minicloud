mod cli_args;
mod fs_object;
mod html_page;

use crate::html_page::html_page;
use axum::{routing::get, Router, extract::{State, Query}, response::Html};
use clap::Parser;
use fs_object::{content_recursively, FSObject};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let mut cli_args = cli_args::Args::parse();
    if cli_args.prepare_data().is_err() {
        return;
    }

    // Get file tree
    let files = match content_recursively(&cli_args.paths) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            return; // close mnicloud
        }
    };

    //println!("{}", html_page::unordered_list(&files));

    //println!("{}", html_page::html_page(&files));

    //----------------------------------------------
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    let app = Router::new()
        .route("/", get(root_handler)
        .with_state(Arc::new(files)));

    axum::serve(listener, app).await.unwrap();
}

async fn root_handler(files: State<Arc<Vec<FSObject>>>) -> Html<String> {
    Html(html_page::html_page(&files))
}
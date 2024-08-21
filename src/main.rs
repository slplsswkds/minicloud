mod cli_args;
mod fs_object;
mod html_page;

use axum::{routing::get, Router, extract::State, response::Html};
use clap::Parser;
use fs_object::content_recursively;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let mut cli_args = cli_args::Args::parse();
    if cli_args.prepare_data().is_err() {
        return;
    }

    if cli_args.paths.is_empty() {
        return;
    }

    // Get file tree
    let fs_objects = match content_recursively(&cli_args.paths) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            return; // close minicloud
        }
    };

    println!("Obtained {} FSObjects", cli_args.paths.len());

    let files_html_page: Html<String>;

    if cli_args.no_html {
        files_html_page = Html("".to_string()); // empty page
    } else {
        print!("Generating HTML...");
        files_html_page = Html(html_page::html_page(&fs_objects));
        println!(" OK");

        #[cfg(debug_assertions)]
        println!("{}", html_page::html_page(&fs_objects));
    }
    //----------------------------------------------
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("\nlistening 127.0.0.1:3000");

    let app = Router::new()
        .route("/", get(root_handler)
            .with_state(Arc::new(files_html_page)),
        );

    axum::serve(listener, app).await.unwrap();
}

async fn root_handler(page: State<Arc<Html<String>>>) -> Html<String> {
    (**page).clone()
}

use std::{net::SocketAddr, env};
use axum::{routing::get, Router};

mod storage;
use storage::{parse_paths, files_show};

mod server;
use server::{root_handler, download_file};

#[tokio::main]
async fn main() {
    let mut args: Vec<String> = env::args().collect(); args.remove(0); args.dedup();
    
    let files = parse_paths(&args);
    if files.len() == 0 { println!("No file has been transferred to the program"); return }
    files_show(&files);

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/download", get(download_file))
        .with_state(files);
    
    let addr = SocketAddr::from(([192, 168, 50, 69], 3005));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
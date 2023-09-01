use std::{net::SocketAddr, env};
use axum::{routing::get, Router};
use local_ip_address::local_ip;

mod storage;
use storage::{parse_paths, files_show};

mod server;
use server::{root_handler, download_file, preview_file};

#[tokio::main]
async fn main() {
    let mut args: Vec<String> = env::args().collect(); args.remove(0); args.dedup();
    
    let files = parse_paths(&args);
    if files.len() == 0 { println!(
        "No file has been transferred to the program

        Use minicloud /path/to/file /other/files/* /more/files/*/*"
    ); return }
    files_show(&files);

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/download", get(download_file))
        .route("/preview", get(preview_file))
        .with_state(files);

    let local_ip = match local_ip() {
        Ok(ip) => ip,
        Err(err) => panic!("Error getting local IP: {:?}", err)
    };
    let socket_addr = SocketAddr::new(local_ip, 48666);
    println!("listening on {}", socket_addr);

    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
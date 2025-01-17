mod cli_args;
mod file_chooser;
mod fs_object;
mod html_page;
mod server_receiver_mode;
mod server_transmitter_mode;
mod storage;

use std::error::Error;
use crate::fs_object::show_fs_objects_summary;
use crate::server_receiver_mode::*;
use crate::server_transmitter_mode::*;
use axum::{extract::DefaultBodyLimit, response::Html, routing::get, Router};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use storage::content_recursively;

use tracing::{debug, info, error, warn};
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

use tower_http::limit::RequestBodyLimitLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let filter = EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();

    let mut cli_args = cli_args::Args::parse();

    let app = if cli_args.receive {
        info!("Receive mode enabled. Files will be saved to: {:?}", cli_args.received_files_path);

        let uploads_path_state = Arc::new(cli_args.received_files_path);

        Router::new()
            .route(
                "/",
                get(show_upload_form)
                    .post(accept_upload_form)
                    .with_state(uploads_path_state),
            )
            .layer(DefaultBodyLimit::disable())
            .layer(RequestBodyLimitLayer::new(
                cli_args.max_received_file_size * 1024 * 1024,
            ))
            .layer(tower_http::trace::TraceLayer::new_for_http())
    } else {
        info!("Transmit mode enabled. Paths: {:?}", cli_args.paths);

        if cli_args.prepare_paths().is_err() {
            return Ok(());
        }

        // Get files tree
        let fs_objects = content_recursively(&cli_args.paths)?;

        // Info about obtained files, directories, and symbolic links
        show_fs_objects_summary(&fs_objects);

        debug!("Generating HTML...");
        let (page, fs_objects_hash_map) = html_page::html_page(&fs_objects);
        debug!("HTML generated.");

        let fs_objects_hash_map_state = Arc::new(fs_objects_hash_map);

        Router::new()
            .route("/", get(root_handler).with_state(Arc::new(Html(page))))
            .route(
                "/dl",
                get(download_handler).with_state(fs_objects_hash_map_state.clone()),
            )
            .route("/pw", get(preview_handler).with_state(fs_objects_hash_map_state))
            .layer(tower_http::trace::TraceLayer::new_for_http())
    };

    //----------------------------------------------

    let local_ip = local_ip_address::local_ip()?;
    let socket_addr = SocketAddr::new(local_ip, cli_args.port);
    let listener = tokio::net::TcpListener::bind(socket_addr);

    info!(
        "Listening on http://{}:{}",
        socket_addr.ip(),
        socket_addr.port()
    );

    axum::serve(listener.await?, app).await?;

    Ok(())
}

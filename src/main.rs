mod cli_args;
mod file_chooser;
mod fs_object;
mod html_page;
mod server;
mod storage;

use crate::fs_object::show_fs_objects_summary;
use crate::server::*;
use axum::{
    extract::DefaultBodyLimit,
    response::Html,
    routing::get,
    Router,
};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use storage::content_recursively;

use tracing::{debug, error, info, trace, warn, Level};
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

use tower_http::limit::RequestBodyLimitLayer;

#[tokio::main]
async fn main() {
    let filter = EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();

    let mut cli_args = cli_args::Args::parse();

    // if cli_args.receive {
    //     println!(
    //         "Receive mode enabled. Files will be saved to: {:?}",
    //         cli_args.received_files_path
    //     );
    //     assert!(cli_args.paths.is_empty(), "Paths should be empty in receive mode");
    // } else {
    //     println!("Transmit mode enabled. Paths: {:?}", cli_args.paths);
    //     assert!(
    //         cli_args.received_files_path.is_none(),
    //         "Received files path should not be set in transmit mode"
    //     );
    // }

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
        if cli_args.paths.is_empty() {
            cli_args.paths = file_chooser::file_chooser_dialog();
        }

        if cli_args.paths.is_empty() {
            return;
        }

        if cli_args.prepare_paths().is_err() {
            return;
        }

        // Get files tree
        let fs_objects = content_recursively(&cli_args.paths).unwrap_or_else(|err| {
            panic!("{err}") // close minicloud
        });

        // Info about obtained files, directories, and symbolic links
        show_fs_objects_summary(&fs_objects);

        debug!("Generating HTML...");
        let (page, hash_map) = html_page::html_page(&fs_objects);
        debug!("HTML generated.");

        let hash_map_state = Arc::new(hash_map);

        Router::new()
            .route("/", get(root_handler).with_state(Arc::new(Html(page))))
            .route(
                "/dl",
                get(download_handler).with_state(hash_map_state.clone()),
            )
            .route("/pw", get(preview_handler).with_state(hash_map_state))
    };

    //----------------------------------------------

    let local_ip = local_ip_address::local_ip().unwrap();
    let socket_addr = SocketAddr::new(local_ip, cli_args.port);
    let listener = tokio::net::TcpListener::bind(socket_addr);

    info!(
        "Listening on http://{}:{}",
        socket_addr.ip(),
        socket_addr.port()
    );

    axum::serve(listener.await.unwrap(), app).await.unwrap();
}

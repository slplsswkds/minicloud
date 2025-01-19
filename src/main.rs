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
        info!("Maximum total files size per request is {} MiB", cli_args.max_total_received_files_size);

        server_receiver_mode::setup(&cli_args)
    } else {
        info!("Transmit mode enabled. Paths: {:?}", cli_args.paths);

        server_transmitter_mode::setup(&mut cli_args)?
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

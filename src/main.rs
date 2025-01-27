mod cli_args;
mod fs_object;
mod html_page;
mod server_receiver_mode;
mod server_transmitter_mode;
mod storage;

use clap::Parser;
use std::net::SocketAddr;

use tracing_subscriber;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();

    let mut cli_args = cli_args::Args::parse();

    let app = if cli_args.receive {
        tracing::info!("Receive mode enabled. Files will be saved to: {:?}", cli_args.received_files_path);
        tracing::info!("Maximum total files size per request is {} MiB", cli_args.max_total_received_files_size);

        server_receiver_mode::setup(&cli_args)
    } else {
        tracing::info!("Transmit mode enabled. Paths: {:?}", cli_args.paths);

        server_transmitter_mode::setup(&mut cli_args)?
    };

    //----------------------------------------------

    let local_ip = local_ip_address::local_ip()?;
    let socket_addr = SocketAddr::new(local_ip, cli_args.port);
    let listener = tokio::net::TcpListener::bind(socket_addr);

    tracing::info!(
        "Listening on http://{}:{}",
        socket_addr.ip(),
        socket_addr.port()
    );

    axum::serve(listener.await?, app).await?;

    Ok(())
}

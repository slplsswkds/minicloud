use crate::cli_args::Args;
use crate::style::STYLE_CSS;
use askama::Template;
use axum::body::{Body, Bytes};
use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::http;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tower_http::limit::RequestBodyLimitLayer;

// static INDEX_HTML: &[u8] = include_bytes!("../templates/server_receiver_mode/index.html");
static SCRIPT_JS: &[u8] = include_bytes!("../templates/server_receiver_mode/script.js");

#[derive(Template)]
#[template(path = "server_receiver_mode/index.html")]
struct ReceiverTemplate {
    title: String,
    max_size: usize,
}

pub async fn show_upload_form(max_total_received_file_size: State<usize>) -> impl IntoResponse {
    tracing::info!("Root page request");

    let page = ReceiverTemplate {
        title: format!("Minicloud v{}", env!("CARGO_PKG_VERSION")),
        max_size: *max_total_received_file_size,
    }
    .render()
    .unwrap();

    axum::response::Html(page)
}

pub fn setup(cli_args: &Args) -> axum::Router {
    tracing::info!(
        "Receive mode enabled. Files will be saved to: {:?}",
        cli_args.received_files_path
    );
    tracing::info!(
        "Maximum total files size per request is {} MiB",
        cli_args.max_total_received_files_size
    );

    let uploads_path_state: Arc<PathBuf> = Arc::new(cli_args.received_files_path.clone().unwrap());
    let max_total_received_files_size = Arc::new(cli_args.max_total_received_files_size);

    axum::Router::new()
        .route(
            "/",
            get(show_upload_form)
                .with_state(*max_total_received_files_size)
                .post(accept_upload_form)
                .with_state(uploads_path_state.clone()),
        )
        // .route("/index.html", get(serve_index_html))
        .route("/script.js", get(serve_script_js))
        .route("/style.css", get(serve_style_css))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            cli_args.max_total_received_files_size * 1024 * 1024,
        ))
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

async fn serve_script_js() -> impl IntoResponse {
    Response::builder()
        .header("Content-Type", "application/javascript")
        .body(Body::from(Bytes::from_static(SCRIPT_JS)))
        .unwrap()
}

async fn serve_style_css() -> impl IntoResponse {
    Response::builder()
        .header("Content-Type", "text/css; charset=utf-8")
        .body(Body::from(Bytes::from_static(STYLE_CSS)))
        .unwrap()
}

pub async fn accept_upload_form(
    uploads_path_state: State<Arc<PathBuf>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let create_dir = tokio::fs::create_dir_all(uploads_path_state.as_ref());

    if create_dir.await.is_err() {
        tracing::error!(
            "Failed to create directory {}",
            uploads_path_state.as_ref().display()
        );
        return Err(http::StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(IntoResponse::into_response)?
    {
        let name = field.name().unwrap_or("unnamed").to_string();
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        let content_type = field.content_type().unwrap_or("unknown-type").to_string();
        let data = field.bytes().await.map_err(|err| {
            tracing::warn!("Failed to read multipart data: {}", err);
            err.into_response()
        })?;

        tracing::debug!(
            r#"Obtained multipart field:
            name: {name}
            file_name: {file_name}
            content_type: {content_type}
            data length: {}
            "#,
            data.len()
        );

        let file_path = uploads_path_state.as_ref().clone().join(file_name);

        let mut file = tokio::fs::File::create(file_path.clone())
            .await
            .map_err(|err| {
                tracing::error!("Failed to create file: {}", err);
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })?;

        file.write_all(&data).await.map_err(|err| {
            tracing::error!("Failed to save file: {}", err);
            http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

        tracing::info!("Received file: {}", file_path.display());
    }

    Ok("Upload successful".into_response())
}

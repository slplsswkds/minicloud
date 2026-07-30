use crate::cli_args::Args;
use crate::style::STYLE_CSS;
use askama::Template;
use axum::{
    extract::{multipart::Field, DefaultBodyLimit, Multipart, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::io::{AsyncWriteExt, BufWriter};
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};

static SCRIPT_JS: &[u8] = include_bytes!("../templates/server_receiver_mode/script.js");
const APP_TITLE: &str = concat!("Minicloud v", env!("CARGO_PKG_VERSION"));

#[derive(Clone)]
pub struct ReceiverState {
    pub uploads_path: Arc<PathBuf>,
    pub max_file_size: usize,
}

#[derive(Template)]
#[template(path = "server_receiver_mode/index.html")]
struct ReceiverTemplate<'a> {
    title: &'a str,
    max_size: usize,
}

pub fn setup(cli_args: &Args) -> Router {
    let uploads_path = cli_args
        .received_files_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("./uploads"));

    if let Err(err) = std::fs::create_dir_all(&uploads_path) {
        tracing::error!(
            "Failed to create uploads directory {:?}: {err}",
            uploads_path
        );
    }

    tracing::info!(
        "Receive mode enabled. Files will be saved to: {:?}",
        uploads_path
    );
    tracing::info!(
        "Maximum total files size per request is {} MiB",
        cli_args.max_total_received_files_size
    );

    let state = ReceiverState {
        uploads_path: Arc::new(uploads_path),
        max_file_size: cli_args.max_total_received_files_size,
    };

    Router::new()
        .route("/", get(show_upload_form).post(accept_upload_form))
        .route("/script.js", get(serve_script_js))
        .route("/style.css", get(serve_style_css))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            cli_args.max_total_received_files_size * 1024 * 1024,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn show_upload_form(
    State(state): State<ReceiverState>,
) -> Result<Html<String>, StatusCode> {
    tracing::info!("Root page request");

    let page = ReceiverTemplate {
        title: APP_TITLE,
        max_size: state.max_file_size,
    }
    .render()
    .map_err(|err| {
        tracing::error!("Template render error: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(page))
}

async fn serve_script_js() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "application/javascript"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        SCRIPT_JS,
    )
}

async fn serve_style_css() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        STYLE_CSS,
    )
}

pub async fn accept_upload_form(
    State(state): State<ReceiverState>,
    mut multipart: Multipart,
) -> Result<&'static str, StatusCode> {
    while let Some(mut field) = multipart.next_field().await.map_err(|err| {
        tracing::warn!("Failed to read multipart field: {err}");
        StatusCode::BAD_REQUEST
    })? {
        let safe_name = sanitize_filename(field.file_name());
        let file_path = state.uploads_path.join(safe_name);

        save_field_to_file(&file_path, &mut field).await?;

        tracing::info!("Received file: {}", file_path.display());
    }

    Ok("Upload successful")
}

async fn save_field_to_file(file_path: &Path, field: &mut Field<'_>) -> Result<(), StatusCode> {
    let tmp_path = file_path.with_extension("tmp");

    let file = tokio::fs::File::create(&tmp_path).await.map_err(|err| {
        tracing::error!("Failed to create temp file {}: {err}", tmp_path.display());
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut writer = BufWriter::new(file);

    while let Some(chunk) = field.chunk().await.map_err(|err| {
        tracing::error!("Failed to read chunk: {err}");
        let _ = tokio::fs::remove_file(&tmp_path); // Видаляємо тимчасовий файл при помилці
        StatusCode::BAD_REQUEST
    })? {
        if let Err(err) = writer.write_all(&chunk).await {
            tracing::error!("Failed to write chunk to file: {err}");
            let _ = tokio::fs::remove_file(&tmp_path);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    if let Err(err) = writer.flush().await {
        tracing::error!("Failed to flush file to disk: {err}");
        let _ = tokio::fs::remove_file(&tmp_path);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    tokio::fs::rename(&tmp_path, file_path)
        .await
        .map_err(|err| {
            tracing::error!("Failed to rename temp file to final destination: {err}");
            let _ = tokio::fs::remove_file(&tmp_path);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(())
}

fn sanitize_filename(raw_name: Option<&str>) -> &str {
    raw_name
        .and_then(|name| name.rsplit('\\').next())
        .and_then(|name| Path::new(name).file_name())
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("unnamed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename(Some("image.png")), "image.png");
        assert_eq!(sanitize_filename(Some("../../etc/passwd")), "passwd");
        assert_eq!(
            sanitize_filename(Some(r"C:\Windows\system32\cmd.exe")),
            "cmd.exe"
        );
        assert_eq!(sanitize_filename(Some("")), "unnamed");
        assert_eq!(sanitize_filename(None), "unnamed");
    }
}

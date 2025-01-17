use crate::cli_args::Args;
use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tower_http::limit::RequestBodyLimitLayer;
use tracing::{error, info, warn};

pub fn setup(cli_args: &Args) -> Router {
    let uploads_path_state = Arc::new(cli_args.received_files_path.clone());
    let max_total_received_files_size = Arc::new(cli_args.max_total_received_files_size);

    Router::new()
        .route(
            "/",
            get(show_upload_form)
                .with_state(*max_total_received_files_size)
                .post(accept_upload_form)
                .with_state(uploads_path_state),
        )
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            cli_args.max_total_received_files_size * 1024 * 1024,
        ))
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

pub async fn show_upload_form(
    max_total_received_file_size: State<usize>
) -> Html<String> {
    info!("Root page request");

    Html(
        format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Upload files</title>
            </head>
            <body>
                <h1>Upload files</h1>
                <p>Maximum total file size: {} MiB </p>
                <form action="/" method="post" enctype="multipart/form-data">
                    <input type="file" name="files" multiple>
                    <button type="submit">Upload</button>
                </form>
            </body>
            </html>
            "#,
            max_total_received_file_size.0
        )
    )
}


pub async fn accept_upload_form(
    uploads_path_state: State<Arc<PathBuf>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let create_dir = tokio::fs::create_dir_all(uploads_path_state.as_ref());

    if create_dir.await.is_err() {
        error!(
            "Failed to create directory {}",
            uploads_path_state.as_ref().display()
        );
        return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
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
            warn!("Failed to read multipart data: {}", err);
            err.into_response()
        })?;

        let file_path = uploads_path_state.as_ref().clone().join(file_name);

        let mut file = tokio::fs::File::create(file_path.clone())
            .await
            .map_err(|err| {
                error!("Failed to create file: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })?;

        file.write_all(&data).await.map_err(|err| {
            error!("Failed to save file: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

        info!("Received file: {}", file_path.display());
    }

    Ok("Upload successful".into_response())
}

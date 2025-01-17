use crate::cli_args::Args;
use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use axum::http::StatusCode;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tower_http::limit::RequestBodyLimitLayer;
use tracing::{error, info};

pub fn setup(cli_args: &Args) -> Router {
    let uploads_path_state = Arc::new(cli_args.received_files_path.clone());

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
}

pub async fn show_upload_form() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Upload Files</title>
        </head>
        <body>
            <h1>Upload Multiple Files</h1>
            <form action="/" method="post" enctype="multipart/form-data">
                <input type="file" name="files" multiple>
                <button type="submit">Upload</button>
            </form>
        </body>
        </html>
        "#,
    )
}

// REMOVE UNWRAPS!!!!!!!!!!!!
pub async fn accept_upload_form(
    uploads_path_state: State<Arc<PathBuf>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let create_dir = tokio::fs::create_dir_all(uploads_path_state.as_ref());

    if create_dir.await.is_err() {
        error!("Failed to create directory {}", uploads_path_state.as_ref().display());
        return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let file_name = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        let file_path = uploads_path_state.as_ref().clone().join(file_name);
        let mut file = tokio::fs::File::create(file_path).await.unwrap();
        file.write_all(&data).await.unwrap();
    }

    Ok("Upload successful".into_response())
}

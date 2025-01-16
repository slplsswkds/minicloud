use crate::fs_object::FsObject;
use axum::extract::Multipart;
use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse},
};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use tokio_util::io::ReaderStream;
use tracing::{info, warn};

pub async fn root_handler(page: State<Arc<Html<String>>>) -> impl IntoResponse {
    info!("Root page request");
    (**page).clone()
}

#[derive(Deserialize)]
pub struct Params {
    id: u64,
}

/// function used to avoid code duplication in download_handler() and preview_handler.
/// Gets an FSObject from a HashMap based on the hash and creates a file read stream
/// and returns the stream and FSObject
async fn prepare_response(
    state: &Arc<HashMap<u64, Arc<FsObject>>>,
    query: &Query<Params>,
) -> Result<(Arc<FsObject>, ReaderStream<tokio::fs::File>), (StatusCode, String)> {
    let fs_object = match state.get(&query.id) {
        Some(fs_obj) => fs_obj.clone(),
        None => {
            warn!("Item not found. ID = {}", &query.id);
            let err_msg = format!("Unexpected error. Item not found. ID = {}", &query.id);
            return Err((StatusCode::NOT_FOUND, err_msg));
        }
    };

    let file = match tokio::fs::File::open(&fs_object.path).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };

    let stream = ReaderStream::new(file);

    Ok((fs_object, stream))
}

pub async fn download_handler(
    state: State<Arc<HashMap<u64, Arc<FsObject>>>>,
    query: Query<Params>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (fs_object, stream) = prepare_response(&state, &query).await?;

    info!("Download request: {}", fs_object.path.display());

    let body = Body::from_stream(stream);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", fs_object.name())
            .parse()
            .unwrap(),
    );

    Ok((StatusCode::OK, headers, body))
}

pub async fn preview_handler(
    state: State<Arc<HashMap<u64, Arc<FsObject>>>>,
    query: Query<Params>,
) -> impl IntoResponse {
    let (fs_object, stream) = prepare_response(&state, &query).await?;

    info!("Preview request: {}", fs_object.path.display());

    let body = Body::from_stream(stream);

    let content_type = match mime_guess::from_path(&fs_object.path).first_raw() {
        Some(mime) => mime,
        None => {
            warn!(
                "Could not preview file: MIME Type couldn't be determined for file: {}",
                fs_object.path.display()
            );
            return Err((
                StatusCode::BAD_REQUEST,
                "MIME Type couldn't be determined".to_string(),
            ));
        }
    };

    let headers = [(header::CONTENT_TYPE, content_type)];

    Ok((StatusCode::OK, headers, body))
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

pub async fn accept_upload_form(mut multipart: Multipart) {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let file_name = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        info!(
            "Obtained: `{name}` (`{file_name}`: `{content_type}`) is {} bytes",
            data.len()
        );
    }
}

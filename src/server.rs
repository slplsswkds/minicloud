use std::{collections::HashMap, sync::Arc};
use axum::{
    extract::{State, Query},
    response::{Html, IntoResponse},
    http::{StatusCode, HeaderMap, header},
    body::Body,
};
use crate::fs_object::FSObject;
use serde::Deserialize;
use tokio_util::io::ReaderStream;

pub async fn root_handler(page: State<Arc<Html<String>>>) -> impl IntoResponse {
    (**page).clone()
}

#[derive(Deserialize)]
pub struct Params {
    id: u64,
}

pub async fn download_handler(
    state: State<Arc<HashMap<u64, Arc<FSObject>>>>,
    query: Query<Params>,
) -> impl IntoResponse {
    let fs_object = match (**state).get(&query.id) {
        Some(fs_obj) => fs_obj,
        None => {
            let err_msg = format!("Unexpected error. Item not found. ID = {}", &query.id);
            return Err((StatusCode::NOT_FOUND, err_msg));
        }
    };

    // `File` implements `AsyncRead`
    let file = match tokio::fs::File::open(&fs_object.path).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };

    // Convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);

    // Convert the `Stream` into an `axum::body::HttpBody`
    let body = Body::from_stream(stream);

    // Create headers for the response
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", fs_object.name())
            .parse()
            .unwrap(),
    );

    // Combine headers and body into a response
    Ok((StatusCode::OK, headers, body))
}

pub async fn preview_handler(
    state: State<Arc<HashMap<u64, Arc<FSObject>>>>,
    query: Query<Params>,
) -> impl IntoResponse {
    let fs_object = match (**state).get(&query.id) {
        Some(fs_obj) => fs_obj,
        None => {
            let err_msg = format!("Unexpected error. Item not found. ID = {}", &query.id);
            return Err((StatusCode::NOT_FOUND, err_msg));
        }
    };

    let content_type = match mime_guess::from_path(&fs_object.path).first_raw() {
        Some(mime) => mime,
        None => return Err((StatusCode::BAD_REQUEST, "MIME Type couldn't be determined".to_string()))
    };

    // `File` implements `AsyncRead`
    let file = match tokio::fs::File::open(&fs_object.path).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };

    // Convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);

    // Convert the `Stream` into an `axum::body::HttpBody`
    let body = Body::from_stream(stream);

    // Create headers for the response

    let headers = [(header::CONTENT_TYPE, content_type)];

    // Combine headers and body into a response
    Ok((StatusCode::OK, headers, body))
}

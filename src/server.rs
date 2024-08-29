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

/// function used to avoid code duplication in download_handler() and preview_handler.
/// Gets an FSObject from a HashMap based on the hash and creates a file read stream
/// and returns the stream and FSObject
async fn prepare_response(
    state: &Arc<HashMap<u64, Arc<FSObject>>>,
    query: &Query<Params>,
) -> Result<(Arc<FSObject>, ReaderStream<tokio::fs::File>), (StatusCode, String)> {
    let fs_object = match state.get(&query.id) {
        Some(fs_obj) => fs_obj.clone(),
        None => {
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
    state: State<Arc<HashMap<u64, Arc<FSObject>>>>,
    query: Query<Params>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (fs_object, stream) = prepare_response(&state, &query).await?;

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
    state: State<Arc<HashMap<u64, Arc<FSObject>>>>,
    query: Query<Params>,
) -> impl IntoResponse {
    let (fs_object, stream) = prepare_response(&state, &query).await?;

    let body = Body::from_stream(stream);

    let content_type = match mime_guess::from_path(&fs_object.path).first_raw() {
        Some(mime) => mime,
        None => return Err((StatusCode::BAD_REQUEST, "MIME Type couldn't be determined".to_string())),
    };

    let headers = [(header::CONTENT_TYPE, content_type)];

    Ok((StatusCode::OK, headers, body))
}

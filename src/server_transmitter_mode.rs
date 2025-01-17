use crate::fs_object::{show_fs_objects_summary, FsObject};
use axum::{body::Body, extract::{Query, State}, http::{header, HeaderMap, StatusCode}, response::{Html, IntoResponse}, Router};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use std::error::Error;
use axum::routing::get;
use tokio_util::io::ReaderStream;
use tracing::{debug, error, info, warn};
use crate::html_page;
use crate::storage::content_recursively;
use crate::cli_args::Args;

pub fn setup(cli_args: &mut Args) -> Result<Router, Box<dyn Error>> {
    cli_args.prepare_paths();

    if cli_args.paths.len() == 0 {
        return Err("No one valid paths provided".into()); // Returning early with an error
    }

    // Get files tree
    let fs_objects = content_recursively(&cli_args.paths)?;

    // Info about obtained files, directories, and symbolic links
    show_fs_objects_summary(&fs_objects);

    debug!("Generating HTML...");
    let (page, fs_objects_hash_map) = html_page::html_page(&fs_objects);
    debug!("HTML generated.");

    let fs_objects_hash_map_state = Arc::new(fs_objects_hash_map);

    let router = Router::new()
        .route("/", get(root_handler).with_state(Arc::new(Html(page))))
        .route(
            "/dl",
            get(download_handler).with_state(fs_objects_hash_map_state.clone()),
        )
        .route("/pw", get(preview_handler).with_state(fs_objects_hash_map_state))
        .layer(tower_http::trace::TraceLayer::new_for_http());

    Ok(router)
}

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
    fs_objects_hash_map_state: State<Arc<HashMap<u64, Arc<FsObject>>>>,
    query: Query<Params>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (fs_object, stream) = prepare_response(&fs_objects_hash_map_state, &query).await?;

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

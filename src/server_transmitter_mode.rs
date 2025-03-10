use crate::cli_args::Args;
use crate::fs_object::{show_fs_objects_summary, FsObject};
use crate::html_page_utils::{unordered_list};
use crate::storage::content_recursively;
use askama::Template;
use axum::routing::get;
use axum::{
    body::Body,
    extract::{Query, State},
    http,
    response::{Html, IntoResponse},
    Router,
};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use tokio_util::io::ReaderStream;

#[derive(Template)]
#[template(path = "server_transmitter_mode/index.html", escape = "none")]
struct TransmitterTemplate {
    title: String,
    files_list: String,
}

async fn show_download_form(page: State<Arc<Html<String>>>) -> impl IntoResponse {
    tracing::info!("Root page request");
    page.as_ref().clone()
}

pub fn setup(cli_args: &mut Args) -> Result<Router, Box<dyn std::error::Error>> {
    tracing::info!("Transmit mode enabled. Paths: {:?}", cli_args.paths);
    cli_args.prepare_paths();

    if cli_args.paths.len() == 0 {
        return Err("No one valid paths provided".into()); // Returning early with an error
    }

    // Get files tree
    let fs_objects = content_recursively(&cli_args.paths)?;

    // Info about obtained files, directories, and symbolic links
    show_fs_objects_summary(&fs_objects);

    tracing::debug!("Generating HTML...");
    let title = format!("Minicloud v{}", env!("CARGO_PKG_VERSION"));

    let mut hash_map = HashMap::new();
    let files_list = unordered_list(&fs_objects, &mut hash_map);

    let page = TransmitterTemplate { title, files_list }.render().unwrap();
    tracing::debug!("HTML generated.");

    let fs_objects_hash_map_state = Arc::new(hash_map);
    println!("{} {}", { "{}" }, page);
    let html_page = Arc::new(Html(page));

    let router = Router::new()
        .route("/", get(show_download_form).with_state(html_page))
        .route(
            "/dl",
            get(download_handler).with_state(fs_objects_hash_map_state.clone()),
        )
        .route(
            "/pw",
            get(preview_handler).with_state(fs_objects_hash_map_state),
        )
        .layer(tower_http::trace::TraceLayer::new_for_http());
    Ok(router)
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
) -> Result<(Arc<FsObject>, ReaderStream<tokio::fs::File>), (http::StatusCode, String)> {
    let fs_object = match state.get(&query.id) {
        Some(fs_obj) => fs_obj.clone(),
        None => {
            tracing::warn!("Item not found. ID = {}", &query.id);
            let err_msg = format!("Unexpected error. Item not found. ID = {}", &query.id);
            return Err((http::StatusCode::NOT_FOUND, err_msg));
        }
    };

    let file = match tokio::fs::File::open(&fs_object.path).await {
        Ok(file) => file,
        Err(err) => {
            return Err((
                http::StatusCode::NOT_FOUND,
                format!("File not found: {}", err),
            ))
        }
    };

    let stream = ReaderStream::new(file);

    Ok((fs_object, stream))
}

pub async fn download_handler(
    fs_objects_hash_map_state: State<Arc<HashMap<u64, Arc<FsObject>>>>,
    query: Query<Params>,
) -> Result<impl IntoResponse, (http::StatusCode, String)> {
    let (fs_object, stream) = prepare_response(&fs_objects_hash_map_state, &query).await?;

    tracing::info!("Download request: {}", fs_object.path.display());

    let body = Body::from_stream(stream);

    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", fs_object.name())
            .parse()
            .unwrap(),
    );

    Ok((http::StatusCode::OK, headers, body))
}

pub async fn preview_handler(
    state: State<Arc<HashMap<u64, Arc<FsObject>>>>,
    query: Query<Params>,
) -> impl IntoResponse {
    let (fs_object, stream) = prepare_response(&state, &query).await?;

    tracing::info!("Preview request: {}", fs_object.path.display());

    let body = Body::from_stream(stream);

    let content_type = match mime_guess::from_path(&fs_object.path).first_raw() {
        Some(mime) => mime,
        None => {
            tracing::warn!(
                "Could not preview file: MIME Type couldn't be determined for file: {}",
                fs_object.path.display()
            );
            return Err((
                http::StatusCode::BAD_REQUEST,
                "MIME Type couldn't be determined".to_string(),
            ));
        }
    };

    let headers = [(http::header::CONTENT_TYPE, content_type)];

    Ok((http::StatusCode::OK, headers, body))
}

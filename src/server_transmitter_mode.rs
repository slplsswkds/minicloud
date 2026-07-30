use crate::cli_args::Args;
use crate::fs_object::{show_fs_objects_summary, FsObject};
use crate::html_page_utils::unordered_list;
use crate::style::STYLE_CSS;
use askama::Template;
use axum::{
    body::Bytes,
    extract::{Query, Request, State},
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use tower::ServiceExt;
use tower_http::{services::ServeFile, trace::TraceLayer};

static SCRIPT_JS: &[u8] = include_bytes!("../templates/server_transmitter_mode/script.js");
const APP_TITLE: &str = concat!("Minicloud v", env!("CARGO_PKG_VERSION"));

#[derive(Clone)]
pub struct TransmitterState {
    pub fs_objects: Arc<HashMap<u64, Arc<FsObject>>>,
    pub index_page: Html<Bytes>,
}

#[derive(Template)]
#[template(path = "server_transmitter_mode/index.html", escape = "none")]
struct TransmitterTemplate<'a> {
    title: &'a str,
    files_list: String,
}

#[derive(Deserialize)]
pub struct Params {
    id: u64,
}

pub fn setup(cli_args: &mut Args) -> Result<Router, Box<dyn std::error::Error>> {
    tracing::info!("Transmit mode enabled. Paths: {:?}", cli_args.paths);
    cli_args.prepare_paths();

    if cli_args.paths.is_empty() {
        return Err("No valid paths provided".into());
    }

    let fs_objects = crate::storage::content_recursively(&cli_args.paths)?;
    show_fs_objects_summary(&fs_objects);

    tracing::debug!("Generating HTML...");

    let mut hash_map = HashMap::new();
    let files_list = unordered_list(&fs_objects, &mut hash_map);

    let html_page = TransmitterTemplate {
        title: APP_TITLE,
        files_list,
    }
    .render()?;

    tracing::debug!("HTML generated.");

    let state = TransmitterState {
        fs_objects: Arc::new(hash_map),
        index_page: Html(Bytes::from(html_page)),
    };

    let router = Router::new()
        .route("/", get(show_download_form))
        .route("/dl", get(download_handler))
        .route("/pw", get(preview_handler))
        .route("/script.js", get(serve_script_js))
        .route("/style.css", get(serve_style_css))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    Ok(router)
}

async fn show_download_form(State(state): State<TransmitterState>) -> impl IntoResponse {
    tracing::info!("Root page request");
    state.index_page
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

pub async fn download_handler(
    State(state): State<TransmitterState>,
    Query(params): Query<Params>,
    request: Request,
) -> Result<Response, StatusCode> {
    let fs_object = state.fs_objects.get(&params.id).ok_or_else(|| {
        tracing::warn!("Download item not found. ID = {}", params.id);
        StatusCode::NOT_FOUND
    })?;

    tracing::info!("Download request: {}", fs_object.path.display());

    let mut response = ServeFile::new(&fs_object.path)
        .oneshot(request)
        .await
        .map_err(|err| {
            tracing::error!("Failed to serve file for download: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_response();

    let raw_name = fs_object.name();
    let ascii_name = raw_name.replace('"', "\\\"");
    let encoded_name = utf8_percent_encode(&raw_name);

    let disposition = format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        ascii_name, encoded_name
    );

    if let Ok(val) = HeaderValue::try_from(disposition) {
        response
            .headers_mut()
            .insert(header::CONTENT_DISPOSITION, val);
    }

    Ok(response)
}

pub async fn preview_handler(
    State(state): State<TransmitterState>,
    Query(params): Query<Params>,
    request: Request,
) -> Result<Response, StatusCode> {
    let fs_object = state.fs_objects.get(&params.id).ok_or_else(|| {
        tracing::warn!("Preview item not found. ID = {}", params.id);
        StatusCode::NOT_FOUND
    })?;

    tracing::info!("Preview request: {}", fs_object.path.display());

    let response = ServeFile::new(&fs_object.path)
        .oneshot(request)
        .await
        .map_err(|err| {
            tracing::error!("Failed to serve file for preview: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_response();

    Ok(response)
}

fn utf8_percent_encode(s: &str) -> String {
    let mut encoded = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

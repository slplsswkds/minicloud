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
    // Ok(Html(format!("required item: {}", fs_object.name()).to_string()));

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
    headers.insert(header::CONTENT_TYPE, "text/toml; charset=utf-8".parse().unwrap());
    headers.insert(header::CONTENT_DISPOSITION, "attachment; filename=\"Cargo.toml\"".parse().unwrap());

    // Combine headers and body into a response
    Ok((StatusCode::OK, headers, body))
}

// pub async fn root_handler(State(files): State<Vec<PathBuf>>) -> Html<String> {
//     let mut list = String::new();
//     list += "<p1> ";
//     for file in files.into_iter() {
//         list += &format!("<a href=\"{}\">{}</a> | <a href=\"{}\">{}</a> </br> ",
//             "/preview?name=".to_string() + &file.to_str().unwrap(),"preview",
//             "/download?name=".to_string() + &file.to_str().unwrap(),file.to_str().unwrap()
//         );
//     }
//     list += " <p1>";
//     return Html(list);
// }

// #[derive(Debug, Deserialize)]
// pub struct Filename {
//     name: Option<String>
// }

// pub async fn download_file(Query(filename): Query<Filename>) -> impl IntoResponse {
//     let filepath = match filename.name {
//         Some(path) => PathBuf::from(&path) ,
//         None => return Err((StatusCode::NOT_FOUND, format!("Missing file name"))),
//     };

//     println!("filepath = {}", &filepath.to_str().unwrap());

//     let file = match tokio::fs::File::open(&filepath).await {
//         Ok(file) => file,
//         Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
//     };
//     let stream: ReaderStream<tokio::fs::File> = ReaderStream::new(file);
//     let body = StreamBody::new(stream);
//     let name = format!("attachment; filename=\"{}\"", filepath.file_name().unwrap().to_str().unwrap());
//     let headers = [(header::CONTENT_DISPOSITION, name)];

//     Ok((headers, body))
// }
//
// pub async fn preview_file(Query(filename): Query<Filename>) -> impl IntoResponse {
//     let filepath = match filename.name {
//         Some(path) => PathBuf::from(&path) ,
//         None => return Err((StatusCode::NOT_FOUND, format!("Missing file name"))),
//     };
//
//     println!("filepath = {}", &filepath.to_str().unwrap());
//
//     let content_type = match mime_guess::from_path(&filepath).first_raw() {
//         Some(mime) => mime,
//         None => return Err((StatusCode::BAD_REQUEST, "MIME Type couldn't be determined".to_string()))
//     };
//
//     let file = match tokio::fs::File::open(&filepath).await {
//         Ok(file) => file,
//         Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
//     };
//     let stream = ReaderStream::new(file);
//     let body = StreamBody::new(stream);
//     let headers = [(header::CONTENT_TYPE, content_type)];
//
//     Ok((headers, body))
// }

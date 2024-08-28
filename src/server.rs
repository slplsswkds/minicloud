use std::{collections::HashMap, sync::Arc};
use axum::{extract::{State, Query}, response::Html};
use crate::fs_object::FSObject;
use serde::Deserialize;

pub async fn root_handler(page: State<Arc<Html<String>>>) -> Html<String> {
    (**page).clone()
}

#[derive(Deserialize)]
pub struct Params {
    id: u64,
}

pub async fn download_handler(
    state: State<Arc<HashMap<u64, Arc<FSObject>>>>,
    query: Query<Params>,
) -> Html<String> {
    match (**state).get(&query.id) {
        Some(fs_obj) => {
            Html(format!("required item: {}", fs_obj.name()).to_string())
        }
        None => Html(format!("Unexpected error. Item not found. ID = {}", &query.id))
    }
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

// pub async fn preview_file(Query(filename): Query<Filename>) -> impl IntoResponse {
//     let filepath = match filename.name {
//         Some(path) => PathBuf::from(&path) ,
//         None => return Err((StatusCode::NOT_FOUND, format!("Missing file name"))),
//     };

//     println!("filepath = {}", &filepath.to_str().unwrap());

//     let content_type = match mime_guess::from_path(&filepath).first_raw() {
//         Some(mime) => mime,
//         None => return Err((StatusCode::BAD_REQUEST, "MIME Type couldn't be determined".to_string()))
//     };

//     let file = match tokio::fs::File::open(&filepath).await {
//         Ok(file) => file,
//         Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
//     };
//     let stream = ReaderStream::new(file);
//     let body = StreamBody::new(stream);
//     let headers = [(header::CONTENT_TYPE, content_type)];

//     Ok((headers, body))
// }

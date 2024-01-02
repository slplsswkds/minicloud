// use std::path::PathBuf;
// use axum::{
//     body::StreamBody,
//     http::{header, StatusCode},
//     response::{Html, IntoResponse},
//     extract::{State, Query},
// };
// use serde::Deserialize;
// use tokio_util::io::ReaderStream;

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

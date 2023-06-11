use std::{
    net::SocketAddr,
    fs::canonicalize,
    env,
    path::{PathBuf},
};
use colored::Colorize;
use axum::{
    body::StreamBody,
    http::{header, StatusCode},
    response::{Html, IntoResponse}, 
    routing::get,
    extract::{State, Query}, 
    Router};
use serde::Deserialize;
use tokio_util::io::ReaderStream;

#[tokio::main]
async fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    args.dedup();
    
    let files = parse_paths(&args);
    if files.len() == 0 { return }
    files_show(&files);

    let state = files;

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/download", get(download_file))
        .with_state(state);

    //let addr = SocketAddr::from(([127, 0, 0, 1], 3005));
    let addr = SocketAddr::from(([192, 168, 50, 69], 3005));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

}

async fn root_handler(State(files): State<Vec<PathBuf>>) -> Html<String> {
    let mut list = String::new();
    list += "<p1> ";
    for file in files.into_iter() {
        list += &format!("<a href=\"{}\">{}</a> </br> ", "/download?name=".to_string() + &file.to_str().unwrap(),file.to_str().unwrap());
    }
    list += " <p1>";
    return Html(list);
}

#[derive(Debug, Deserialize)]
struct Filename {
    name: Option<String>
}

async fn download_file(Query(filename): Query<Filename>) -> impl IntoResponse {
    let filepath = match filename.name {
        Some(path) => PathBuf::from(&path) ,
        None => return Err((StatusCode::NOT_FOUND, format!("Missing file name"))),
    };

    println!("filepath = {}", &filepath.to_str().unwrap());

    let content_type = match mime_guess::from_path(&filepath).first_raw() {
        Some(mime) => mime,
        None => return Err((StatusCode::BAD_REQUEST, "MIME Type couldn't be determined".to_string()))
    };

    let file = match tokio::fs::File::open(&filepath).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };
    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);
    let name = &format!("attachment; filename=\"{:?}\"", filepath.file_name());
    let headers = [
        (header::CONTENT_TYPE, content_type),
        //(header::CONTENT_DISPOSITION, "attachment; filename=\"file.mp3\""),
    ];

    Ok((headers, body))
}

fn parse_paths(str_path: &Vec<String>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    
    for arg in str_path {
        let path = PathBuf::from(&arg);
        if path.exists() {
            let full_path = canonicalize(&path).unwrap();
            files.push(full_path);
        }
        else if path.is_dir() {
            println!("\"{}\" is directory. This is not yet supported. Ignoring the argument...", arg.yellow());
        }
        else {
            println!("File \"{}\" not found. Ignoring the argument...", arg.yellow());
        }
    }
    files.dedup();
    return files;
}

fn files_show(files: &Vec<PathBuf>) {
    for file in files.into_iter() {
        println!("{}", file.to_str().unwrap().cyan());
    }
}
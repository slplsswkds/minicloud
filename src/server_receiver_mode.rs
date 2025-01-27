use crate::cli_args::Args;
use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::http;
use axum::response::IntoResponse;
use axum::routing::get;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tower_http::limit::RequestBodyLimitLayer;

pub fn setup(cli_args: &Args) -> axum::Router {
    let uploads_path_state: Arc<PathBuf> = Arc::new(cli_args.received_files_path.clone().unwrap());
    let max_total_received_files_size = Arc::new(cli_args.max_total_received_files_size);

    axum::Router::new()
        .route(
            "/",
            get(show_upload_form)
                .with_state(*max_total_received_files_size)
                .post(accept_upload_form)
                .with_state(uploads_path_state),
        )
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            cli_args.max_total_received_files_size * 1024 * 1024,
        ))
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

pub async fn show_upload_form(max_total_received_file_size: State<usize>) -> impl IntoResponse {
    tracing::info!("Root page request");

    //let html_page: String = HtmlPage::new().with_title().to_html_string();

    axum::response::Html(format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Upload files</title>
            <style>
                ul {{ list-style-type: none; padding: 0; }}
                li {{ margin: 5px 0; }}
            </style>
        </head>
        <body>
            <h1>Upload files</h1>
            <p>Maximum total file size: {} MiB</p>
            <form action="/" method="post" enctype="multipart/form-data" id="upload-form">
                <input type="file" id="file-input" multiple>
                <ul id="file-list"></ul>
                <button type="submit" id="upload-button" disabled>Upload</button>
            </form>

            <script>
                const fileInput = document.getElementById('file-input');
                const fileList = document.getElementById('file-list');
                const uploadButton = document.getElementById('upload-button');

                let filesArray = [];

                // Event listener to handle file selection
                fileInput.addEventListener('change', (event) => {{
                    // Add selected files to the array
                    for (const file of event.target.files) {{
                        filesArray.push(file);
                    }}

                    // Update the displayed file list
                    updateFileList();

                    // Enable the upload button if there are files in the array
                    uploadButton.disabled = filesArray.length === 0;

                    // Clear the file input value to allow re-selecting the same files
                    fileInput.value = "";
                }});

                // Function to update the visual file list
                function updateFileList() {{
                    // Clear the current list in the HTML
                    fileList.innerHTML = "";

                    // Add each file to the visual list
                    filesArray.forEach((file, index) => {{
                        const li = document.createElement('li');
                        li.textContent = `${{file.name}} (${{(file.size / 1024).toFixed(2)}} KB)`;

                        // Add a remove button for each file
                        const removeButton = document.createElement('button');
                        removeButton.textContent = "Remove";
                        removeButton.style.marginLeft = "10px";
                        removeButton.addEventListener('click', () => {{
                            // Remove the file from the array
                            filesArray.splice(index, 1);
                            updateFileList();
                            // Disable the upload button if no files remain
                            uploadButton.disabled = filesArray.length === 0;
                        }});

                        li.appendChild(removeButton);
                        fileList.appendChild(li);
                    }});
                }}

                // Handle form submission and send files using FormData
                document.getElementById('upload-form').addEventListener('submit', (event) => {{
                    event.preventDefault(); // Prevent default form submission
                    const formData = new FormData();

                    // Append each file to FormData
                    filesArray.forEach(file => {{
                        formData.append('files', file);
                    }});

                    // Use Fetch API to send the form data
                    fetch("/", {{
                        method: "POST",
                        body: formData
                    }})
                    .then(response => {{
                        if (response.ok) {{
                            alert("Files uploaded successfully!");
                            // Clear the file array and update the list
                            filesArray = [];
                            updateFileList();
                            uploadButton.disabled = true;
                        }} else {{
                            alert("Failed to upload files.");
                        }}
                    }})
                    .catch(error => {{
                        console.error("Error:", error);
                        alert("An error occurred while uploading files.");
                    }});
                }});
            </script>
        </body>
        </html>
    "#,
        max_total_received_file_size.0
    ))


}

pub async fn accept_upload_form(
    uploads_path_state: State<Arc<PathBuf>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let create_dir = tokio::fs::create_dir_all(uploads_path_state.as_ref());

    if create_dir.await.is_err() {
        tracing::error!(
            "Failed to create directory {}",
            uploads_path_state.as_ref().display()
        );
        return Err(http::StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(IntoResponse::into_response)?
    {
        let name = field.name().unwrap_or("unnamed").to_string();
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        let content_type = field.content_type().unwrap_or("unknown-type").to_string();
        let data = field.bytes().await.map_err(|err| {
            tracing::warn!("Failed to read multipart data: {}", err);
            err.into_response()
        })?;

        tracing::debug!(
            r#"Obtained multipart field:
            name: {name}
            file_name: {file_name}
            content_type: {content_type}
            data length: {}
            "#, data.len()
        );

        let file_path = uploads_path_state.as_ref().clone().join(file_name);

        let mut file = tokio::fs::File::create(file_path.clone())
            .await
            .map_err(|err| {
                tracing::error!("Failed to create file: {}", err);
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })?;

        file.write_all(&data).await.map_err(|err| {
            tracing::error!("Failed to save file: {}", err);
            http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

        tracing::info!("Received file: {}", file_path.display());
    }

    Ok("Upload successful".into_response())
}

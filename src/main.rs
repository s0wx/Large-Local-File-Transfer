use std::{net::SocketAddr, path::PathBuf};
use axum::{
    extract::{
        DefaultBodyLimit,
        Multipart
    },
    http::StatusCode,
    response::{
        Html,
        IntoResponse
    },
    routing::{get},
    Router
};
use axum_server::tls_rustls::RustlsConfig;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use log::{error};

const UPLOAD_FOLDER: &str = "./uploads";


// HTML form handler
async fn upload_form() -> Html<&'static str> {
    Html(r#"
        <!doctype html>
        <title>Upload File</title>
        <h1>Upload File</h1>
        <form method="post" enctype="multipart/form-data">
            <input type="file" name="file">
            <input type="submit" value="Upload">
        </form>
    "#)
}

// File upload handler
async fn upload_file(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let filename = match field.file_name() {
            Some(f) => f.to_string(),
            None => "file.bin".to_string(),
        };

        let filepath = PathBuf::from(UPLOAD_FOLDER).join(filename);

        match File::create(&filepath).await {
            Ok(mut file) => {
                while let Ok(Some(chunk)) = field.chunk().await {
                    if let Err(e) = file.write_all(&chunk).await {
                        error!("Failed to write chunk: {}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to write file",
                        );
                    }
                }

                println!("Successfully uploaded: {}", filepath.display());
                
                (
                    StatusCode::OK,
                    "File uploaded successfully."
                );
            }
            Err(e) => {
                error!("Failed to create file: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to create file on server",
                );
            }
        }
    }

    (StatusCode::BAD_REQUEST, "No file uploaded")
}

#[tokio::main]
async fn main() {
    // Create the "uploads" directory if it doesn't exist
    if !PathBuf::from(UPLOAD_FOLDER).exists() {
        fs::create_dir_all(UPLOAD_FOLDER).await.unwrap();
    }

    // Set up routes
    let app = Router::new()
        .route("/", get(upload_form).post(upload_file))
        .layer(DefaultBodyLimit::disable());

    // Load TLS config
    let tls_config = match RustlsConfig::from_pem_file("cert.pem", "key.pem").await {
        Ok(config) => config,
        Err(e) => {
            println!("Failed to load TLS config: {}", e);
            error!("Failed to load TLS config: {}", e);
            return;
        }
    };
    
    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 5555));
    println!("Listening on https://{}", addr);

    // Serve with TLS
    if let Err(e) = axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
    {
        error!("Server error: {}", e);
    }
    
    // No TLS
    // axum_server::bind(addr)
    //     .serve(app.into_make_service())
    //     .await
    //     .unwrap();
}

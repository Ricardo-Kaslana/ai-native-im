/// Simplified Media API for local file storage
/// For development use only
use axum::{
    extract::{Extension, Path as AxumPath, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use tokio::fs;

use crate::middleware::jwt::UserContext;

/// Media state for local file storage
#[derive(Clone)]
pub struct MediaState {
    pub db_pool: PgPool,
    pub storage_path: String,
    pub base_url: String,
}

/// Upload response
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub media_id: String,
    pub file_url: String,
}

/// Base64 upload request
#[derive(Debug, Deserialize)]
pub struct Base64UploadRequest {
    pub file_name: String,
    pub mime_type: String,
    pub data: String, // base64 without data:url prefix
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

/// Create public media routes (no authentication required)
pub fn public_media_routes() -> Router<MediaState> {
    Router::new()
        .route("/avatars/:filename", get(get_avatar))
        .route("/uploads/:filename", get(get_upload))
}

/// Create protected media routes (authentication required)
pub fn protected_media_routes() -> Router<MediaState> {
    Router::new()
        .route("/upload/base64", post(upload_base64))
}

/// GET /media/avatars/:filename - Get avatar file
async fn get_avatar(
    State(state): State<MediaState>,
    AxumPath(filename): AxumPath<String>,
) -> Result<Response, ErrorResponse> {
    let file_path = format!("{}/avatars/{}", state.storage_path, filename);

    if let Ok(_metadata) = fs::metadata(&file_path).await {
        let mime_type = mime_guess::from_path(&file_path)
            .first_or_octet_stream()
            .to_string();

        let contents = match fs::read(&file_path).await {
            Ok(c) => c,
            Err(e) => {
                return Err(ErrorResponse {
                    error: format!("Failed to read file: {}", e),
                    code: "READ_ERROR".to_string(),
                });
            }
        };

        return Ok((
            [(axum::http::header::CONTENT_TYPE, mime_type)],
            contents,
        ).into_response());
    }

    Err(ErrorResponse {
        error: "File not found".to_string(),
        code: "NOT_FOUND".to_string(),
    })
}

/// GET /media/uploads/:filename - Get upload file
async fn get_upload(
    State(state): State<MediaState>,
    AxumPath(filename): AxumPath<String>,
) -> Result<Response, ErrorResponse> {
    let file_path = format!("{}/uploads/{}", state.storage_path, filename);

    if let Ok(_metadata) = fs::metadata(&file_path).await {
        let mime_type = mime_guess::from_path(&file_path)
            .first_or_octet_stream()
            .to_string();

        let contents = match fs::read(&file_path).await {
            Ok(c) => c,
            Err(e) => {
                return Err(ErrorResponse {
                    error: format!("Failed to read file: {}", e),
                    code: "READ_ERROR".to_string(),
                });
            }
        };

        return Ok((
            [(axum::http::header::CONTENT_TYPE, mime_type)],
            contents,
        ).into_response());
    }

    Err(ErrorResponse {
        error: "File not found".to_string(),
        code: "NOT_FOUND".to_string(),
    })
}

/// POST /media/upload/base64 - Upload base64 encoded image
async fn upload_base64(
    State(state): State<MediaState>,
    _user: Extension<UserContext>,
    Json(req): Json<Base64UploadRequest>,
) -> Result<Json<UploadResponse>, ErrorResponse> {
    use base64::{engine::general_purpose::STANDARD, Engine};

    // Decode base64
    let file_data = match STANDARD.decode(&req.data) {
        Ok(data) => data,
        Err(e) => {
            return Err(ErrorResponse {
                error: format!("Invalid base64: {}", e),
                code: "INVALID_BASE64".to_string(),
            });
        }
    };

    // Generate unique filename
    let media_id = Uuid::new_v4();
    let extension = get_extension(&req.mime_type);
    let filename = format!("{}.{}", media_id, extension);

    // Ensure directory exists
    let dir_path = format!("{}/avatars", state.storage_path);
    if let Err(e) = fs::create_dir_all(&dir_path).await {
        tracing::error!("Failed to create directory: {}", e);
        return Err(ErrorResponse {
            error: format!("Failed to create directory: {}", e),
            code: "STORAGE_ERROR".to_string(),
        });
    }

    // Write file
    let file_path = format!("{}/avatars/{}", state.storage_path, filename);
    if let Err(e) = fs::write(&file_path, file_data).await {
        tracing::error!("Failed to write file: {}", e);
        return Err(ErrorResponse {
            error: format!("Failed to write file: {}", e),
            code: "STORAGE_ERROR".to_string(),
        });
    }

    let file_url = format!("{}/media/avatars/{}", state.base_url, filename);

    tracing::info!("Uploaded base64 file: {}", file_url);

    Ok(Json(UploadResponse {
        media_id: media_id.to_string(),
        file_url,
    }))
}

fn get_extension(mime_type: &str) -> String {
    match mime_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "bin",
    }.to_string()
}

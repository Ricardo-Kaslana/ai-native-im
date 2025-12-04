use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::jwt::UserContext;
use crate::services::media::{
    DeduplicationManager, ImageProcessor, MediaError, MediaFile,
    MediaStatus, MediaType, StorageManager, UploadHandler, VideoProcessor,
};

/// Application state for media routes
#[derive(Clone)]
pub struct MediaState {
    pub db_pool: PgPool,
    pub upload_handler: Arc<UploadHandler>,
    pub image_processor: Arc<ImageProcessor>,
    pub video_processor: Arc<VideoProcessor>,
    pub storage_manager: Arc<StorageManager>,
    pub dedup_manager: Arc<DeduplicationManager>,
}

/// Request to upload an image
#[derive(Debug, Deserialize)]
pub struct ImageUploadRequest {
    pub file_size: u64,
    pub mime_type: String,
}

/// Request to upload a video
#[derive(Debug, Deserialize)]
pub struct VideoUploadRequest {
    pub file_size: u64,
    pub mime_type: String,
}

/// Request to complete an upload
#[derive(Debug, Deserialize)]
pub struct CompleteUploadRequest {
    pub media_id: Uuid,
    pub file_hash: String,
}

/// Response for upload request
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub media_id: Uuid,
    pub upload_url: String,
    pub expires_at: i64,
}

/// Response for media info
#[derive(Debug, Serialize)]
pub struct MediaResponse {
    pub id: Uuid,
    pub media_type: String,
    pub mime_type: String,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<f64>,
    pub file_size: i64,
    pub status: String,
    pub created_at: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

/// Create media routes
pub fn media_routes() -> Router<MediaState> {
    Router::new()
        .route("/upload/image", post(request_image_upload))
        .route("/upload/video", post(request_video_upload))
        .route("/complete", post(complete_upload))
        .route("/:id", get(get_media_info))
        .route("/:id/thumbnail", get(get_thumbnail))
        .route("/:id", delete(delete_media))
}

/// POST /media/upload/image - Request image upload
async fn request_image_upload(
    State(state): State<MediaState>,
    Extension(user): Extension<UserContext>,
    Json(req): Json<ImageUploadRequest>,
) -> Result<Json<UploadResponse>, MediaApiError> {
    let token = state
        .upload_handler
        .request_image_upload(user.user_id, req.file_size, req.mime_type)
        .await?;

    Ok(Json(UploadResponse {
        media_id: token.media_id,
        upload_url: token.upload_url,
        expires_at: token.expires_at,
    }))
}

/// POST /media/upload/video - Request video upload
async fn request_video_upload(
    State(state): State<MediaState>,
    Extension(user): Extension<UserContext>,
    Json(req): Json<VideoUploadRequest>,
) -> Result<Json<UploadResponse>, MediaApiError> {
    let token = state
        .upload_handler
        .request_video_upload(user.user_id, req.file_size, req.mime_type)
        .await?;

    Ok(Json(UploadResponse {
        media_id: token.media_id,
        upload_url: token.upload_url,
        expires_at: token.expires_at,
    }))
}

/// POST /media/complete - Complete upload
async fn complete_upload(
    State(state): State<MediaState>,
    Extension(_user): Extension<UserContext>,
    Json(req): Json<CompleteUploadRequest>,
) -> Result<Json<MediaResponse>, MediaApiError> {

    // Check for duplicate
    if let Some(existing_id) = state.dedup_manager.check_duplicate(&req.file_hash).await? {
        // Increment reference count for existing file
        state.dedup_manager.increment_ref_count(existing_id).await?;

        // Get existing media info
        let media = get_media_by_id(&state.db_pool, existing_id).await?;
        return Ok(Json(media_to_response(&media)));
    }

    // Complete the upload
    let media_id = state
        .upload_handler
        .complete_upload(req.media_id, req.file_hash.clone())
        .await?;

    // Register the file hash
    state
        .dedup_manager
        .register_file(&req.file_hash, media_id)
        .await?;

    // Get media info
    let media = get_media_by_id(&state.db_pool, media_id).await?;

    Ok(Json(media_to_response(&media)))
}

/// GET /media/:id - Get media info
async fn get_media_info(
    State(state): State<MediaState>,
    Path(media_id): Path<Uuid>,
) -> Result<Json<MediaResponse>, MediaApiError> {
    let media = get_media_by_id(&state.db_pool, media_id).await?;
    Ok(Json(media_to_response(&media)))
}

/// GET /media/:id/thumbnail - Get thumbnail URL
async fn get_thumbnail(
    State(state): State<MediaState>,
    Path(media_id): Path<Uuid>,
) -> Result<Json<ThumbnailResponse>, MediaApiError> {
    let media = get_media_by_id(&state.db_pool, media_id).await?;

    let thumbnail_url = media.thumbnail_url.ok_or_else(|| {
        MediaApiError::NotFound("Thumbnail not available".to_string())
    })?;

    Ok(Json(ThumbnailResponse { thumbnail_url }))
}

#[derive(Debug, Serialize)]
pub struct ThumbnailResponse {
    pub thumbnail_url: String,
}

/// DELETE /media/:id - Delete media
async fn delete_media(
    State(state): State<MediaState>,
    Extension(user): Extension<UserContext>,
    Path(media_id): Path<Uuid>,
) -> Result<StatusCode, MediaApiError> {
    let user_id = user.user_id;

    // Get media info
    let media = get_media_by_id(&state.db_pool, media_id).await?;

    // Verify ownership
    if media.uploader_id != user_id {
        return Err(MediaApiError::Unauthorized);
    }

    // Decrement reference count
    let should_delete = state.dedup_manager.decrement_ref_count(media_id).await?;

    if should_delete {
        // Delete from storage
        let is_image = matches!(media.media_type, MediaType::Image);

        // Extract key from URL (simplified - in production, store key separately)
        if let Some(key) = extract_key_from_url(&media.original_url) {
            let _ = state.storage_manager.delete_file(&key, is_image).await;
        }

        // Delete processed and thumbnail files
        if let Some(processed_url) = &media.processed_url {
            if let Some(key) = extract_key_from_url(processed_url) {
                let _ = state.storage_manager.delete_file(&key, is_image).await;
            }
        }

        if let Some(thumbnail_url) = &media.thumbnail_url {
            if let Some(key) = extract_key_from_url(thumbnail_url) {
                let _ = state.storage_manager.delete_file(&key, is_image).await;
            }
        }

        // Delete from database
        state.dedup_manager.delete_file_record(media_id).await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

// Helper functions

async fn get_media_by_id(pool: &PgPool, media_id: Uuid) -> Result<MediaFile, MediaApiError> {
    let media = sqlx::query_as!(
        MediaFile,
        r#"
        SELECT 
            id, uploader_id, file_hash, 
            media_type as "media_type: MediaType",
            mime_type, storage_provider, original_url, processed_url, thumbnail_url,
            width, height, duration::float8 as "duration: f64", file_size, ref_count,
            status as "status: MediaStatus",
            created_at, updated_at
        FROM media_files
        WHERE id = $1
        "#,
        media_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| MediaApiError::Internal(e.to_string()))?
    .ok_or_else(|| MediaApiError::NotFound("Media not found".to_string()))?;

    Ok(media)
}

fn media_to_response(media: &MediaFile) -> MediaResponse {
    MediaResponse {
        id: media.id,
        media_type: match media.media_type {
            MediaType::Image => "image".to_string(),
            MediaType::Video => "video".to_string(),
        },
        mime_type: media.mime_type.clone(),
        url: media.processed_url.clone().unwrap_or_else(|| media.original_url.clone()),
        thumbnail_url: media.thumbnail_url.clone(),
        width: media.width,
        height: media.height,
        duration: media.duration,
        file_size: media.file_size,
        status: match media.status {
            MediaStatus::Processing => "processing".to_string(),
            MediaStatus::Completed => "completed".to_string(),
            MediaStatus::Failed => "failed".to_string(),
        },
        created_at: media.created_at.to_rfc3339(),
    }
}

fn extract_key_from_url(url: &str) -> Option<String> {
    // Simple extraction - assumes URL format like https://bucket.s3.region.amazonaws.com/key
    // or https://endpoint/bucket/key
    url.split('/').skip(3).collect::<Vec<_>>().join("/").into()
}

// Error handling

#[derive(Debug)]
pub enum MediaApiError {
    BadRequest(String),
    Unauthorized,
    NotFound(String),
    Internal(String),
}

impl From<MediaError> for MediaApiError {
    fn from(err: MediaError) -> Self {
        match err {
            MediaError::FileSizeTooLarge(_, _) => MediaApiError::BadRequest(err.to_string()),
            MediaError::UnsupportedMimeType(_) => MediaApiError::BadRequest(err.to_string()),
            MediaError::InvalidFormat(_) => MediaApiError::BadRequest(err.to_string()),
            MediaError::FileNotFound(_) => MediaApiError::NotFound(err.to_string()),
            MediaError::InvalidUploadToken => MediaApiError::BadRequest(err.to_string()),
            MediaError::Unauthorized => MediaApiError::Unauthorized,
            MediaError::FileStillReferenced => MediaApiError::BadRequest(err.to_string()),
            _ => MediaApiError::Internal(err.to_string()),
        }
    }
}

impl IntoResponse for MediaApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error, code) = match self {
            MediaApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg, "BAD_REQUEST"),
            MediaApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized".to_string(),
                "UNAUTHORIZED",
            ),
            MediaApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg, "NOT_FOUND"),
            MediaApiError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                msg,
                "INTERNAL_ERROR",
            ),
        };

        let body = Json(ErrorResponse {
            error,
            code: code.to_string(),
        });

        (status, body).into_response()
    }
}

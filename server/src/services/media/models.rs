use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Media type enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum MediaType {
    #[sqlx(rename = "image")]
    Image,
    #[sqlx(rename = "video")]
    Video,
}

/// Media processing status
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum MediaStatus {
    #[sqlx(rename = "processing")]
    Processing,
    #[sqlx(rename = "completed")]
    Completed,
    #[sqlx(rename = "failed")]
    Failed,
}

/// Media file record in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MediaFile {
    pub id: Uuid,
    pub uploader_id: Uuid,
    pub file_hash: String,
    pub media_type: MediaType,
    pub mime_type: String,
    pub storage_provider: String,
    pub original_url: String,
    pub processed_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<f64>,
    pub file_size: i64,
    pub ref_count: i32,
    pub status: MediaStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Upload token for presigned URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadToken {
    pub token: String,
    pub upload_url: String,
    pub expires_at: i64,
    pub media_id: Uuid,
}

/// Processed image result
#[derive(Debug, Clone)]
pub struct ProcessedImage {
    pub original_path: String,
    pub webp_path: String,
    pub thumbnail_path: String,
    pub width: u32,
    pub height: u32,
    pub file_size: u64,
}

/// Processed video result
#[derive(Debug, Clone)]
pub struct ProcessedVideo {
    pub original_path: String,
    pub compressed_path: String,
    pub thumbnail_path: String,
    pub width: u32,
    pub height: u32,
    pub duration: f64,
    pub file_size: u64,
}

/// Media metadata
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MediaMetadata {
    pub media_id: Uuid,
    pub format: Option<String>,
    pub codec: Option<String>,
    pub bitrate: Option<i32>,
    pub framerate: Option<f64>,
    pub metadata: Option<serde_json::Value>,
}

/// Upload request
#[derive(Debug, Deserialize)]
pub struct UploadRequest {
    pub file_size: u64,
    pub mime_type: String,
}

/// Upload completion request
#[derive(Debug, Deserialize)]
pub struct CompleteUploadRequest {
    pub token: String,
    pub file_hash: String,
}

/// Media info response
#[derive(Debug, Serialize)]
pub struct MediaInfoResponse {
    pub id: Uuid,
    pub media_type: MediaType,
    pub mime_type: String,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<f64>,
    pub file_size: i64,
    pub status: MediaStatus,
    pub created_at: DateTime<Utc>,
}

impl From<MediaFile> for MediaInfoResponse {
    fn from(file: MediaFile) -> Self {
        Self {
            id: file.id,
            media_type: file.media_type,
            mime_type: file.mime_type,
            url: file.processed_url.unwrap_or(file.original_url),
            thumbnail_url: file.thumbnail_url,
            width: file.width,
            height: file.height,
            duration: file.duration,
            file_size: file.file_size,
            status: file.status,
            created_at: file.created_at,
        }
    }
}

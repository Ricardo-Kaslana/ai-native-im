use thiserror::Error;

#[derive(Error, Debug)]
pub enum MediaError {
    #[error("File size exceeds limit: {0} bytes (max: {1} bytes)")]
    FileSizeTooLarge(u64, u64),

    #[error("Unsupported MIME type: {0}")]
    UnsupportedMimeType(String),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Upload token expired or invalid")]
    InvalidUploadToken,

    #[error("Malware detected in file: {0}")]
    MalwareDetected(String),

    #[error("Malware scan failed: {0}")]
    ScanFailed(String),

    #[error("Malware scanner unavailable")]
    ScannerUnavailable,

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("File is still referenced by messages")]
    FileStillReferenced,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Image processing error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, MediaError>;

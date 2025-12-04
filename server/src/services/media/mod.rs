// Media Processing System
//
// This module handles all media file operations including:
// - Image and video upload
// - Format conversion and compression
// - Thumbnail generation
// - Storage management (S3 + CDN)
// - Deduplication
// - Lifecycle management
// - Malware scanning

pub mod config;
pub mod upload_handler;
pub mod image_processor;
pub mod video_processor;
pub mod storage_manager;
pub mod dedup_manager;
pub mod processing_queue;
pub mod malware_scanner;
pub mod lifecycle_manager;
pub mod metadata_manager;
pub mod access_control;
pub mod metrics;
pub mod models;
pub mod error;
pub mod providers;

pub use config::MediaConfig;
pub use upload_handler::UploadHandler;
pub use image_processor::{ImageProcessor, ImageProcessorConfig, ImageMetadata};
pub use video_processor::{VideoProcessor, VideoProcessorConfig, VideoMetadata, VideoCodec};
pub use storage_manager::{StorageManager, StorageProvider, StorageProviderType};
pub use dedup_manager::{DeduplicationManager, DeletedFileInfo};
pub use processing_queue::{ProcessingQueue, ProcessingTask, QueueStats};
pub use malware_scanner::{
    MalwareScannerManager, MalwareScannerConfig, MalwareScanner,
    ScanResult, ScanStatus, ScannerType, PendingScan,
};
pub use lifecycle_manager::{
    LifecycleManager, LifecycleConfig, LifecycleStatus,
    LifecycleCleanupResult, LifecycleStats,
};
pub use metadata_manager::{
    MetadataManager, MediaMetadata, MediaInfo, MetadataUpdate,
};
pub use access_control::{AccessControlManager, DeletePermission};
pub use metrics::{MediaLogger, MediaMetrics, MediaEvent, MediaEventType, MetricsSnapshot};
pub use models::*;
pub use error::MediaError;
pub use providers::*;

#[cfg(test)]
mod tests;

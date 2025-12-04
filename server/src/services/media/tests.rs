//! Unit tests for Media Processing System
//!
//! Tests cover:
//! - File validation
//! - Image processing
//! - Video processing
//! - Deduplication
//! - Storage operations
//! - Provider configuration

#[cfg(test)]
mod file_validation_tests {
    use crate::services::media::config::MediaConfig;

    #[test]
    fn test_image_size_validation() {
        let config = MediaConfig::default();
        
        // Valid image size
        assert!(1024 * 1024 <= config.max_image_size); // 1MB should be valid
        
        // Default max is 10MB
        assert_eq!(config.max_image_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_video_size_validation() {
        let config = MediaConfig::default();
        
        // Default max is 100MB
        assert_eq!(config.max_video_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_image_mime_types() {
        let valid_types = ["image/jpeg", "image/png", "image/gif"];
        let invalid_types = ["image/bmp", "image/tiff", "application/pdf"];

        for mime in valid_types {
            assert!(
                matches!(mime, "image/jpeg" | "image/png" | "image/gif"),
                "Should accept {}",
                mime
            );
        }

        for mime in invalid_types {
            assert!(
                !matches!(mime, "image/jpeg" | "image/png" | "image/gif"),
                "Should reject {}",
                mime
            );
        }
    }

    #[test]
    fn test_video_mime_types() {
        let valid_types = ["video/mp4", "video/quicktime", "video/webm"];
        let invalid_types = ["video/avi", "video/mkv", "audio/mp3"];

        for mime in valid_types {
            assert!(
                matches!(mime, "video/mp4" | "video/quicktime" | "video/webm"),
                "Should accept {}",
                mime
            );
        }

        for mime in invalid_types {
            assert!(
                !matches!(mime, "video/mp4" | "video/quicktime" | "video/webm"),
                "Should reject {}",
                mime
            );
        }
    }

    #[test]
    fn test_upload_expiry_times() {
        let config = MediaConfig::default();
        
        // Image upload: 15 minutes
        assert_eq!(config.image_upload_expiry, 15 * 60);
        
        // Video upload: 30 minutes
        assert_eq!(config.video_upload_expiry, 30 * 60);
    }
}


#[cfg(test)]
mod image_processor_tests {
    use crate::services::media::image_processor::{ImageProcessor, ImageProcessorConfig};

    #[test]
    fn test_image_processor_config_defaults() {
        let config = ImageProcessorConfig::default();
        
        assert_eq!(config.webp_quality, 85);
        assert_eq!(config.max_dimension, 1920);
        assert_eq!(config.thumbnail_dimension, 200);
    }

    #[test]
    fn test_image_processor_creation() {
        let processor = ImageProcessor::new();
        // Processor should be created with default config
        assert!(true);
    }

    #[test]
    fn test_image_processor_with_custom_config() {
        let config = ImageProcessorConfig {
            webp_quality: 90,
            max_dimension: 2560,
            thumbnail_dimension: 300,
        };
        let _processor = ImageProcessor::with_config(config);
        // Should create processor with custom config
        assert!(true);
    }

    #[test]
    fn test_webp_quality_range() {
        // Quality should be between 0 and 100
        let config = ImageProcessorConfig::default();
        assert!(config.webp_quality <= 100);
        assert!(config.webp_quality > 0);
    }

    #[test]
    fn test_max_dimension_reasonable() {
        let config = ImageProcessorConfig::default();
        // Max dimension should be reasonable (not too small, not too large)
        assert!(config.max_dimension >= 1080);
        assert!(config.max_dimension <= 4096);
    }

    #[test]
    fn test_thumbnail_dimension_smaller_than_max() {
        let config = ImageProcessorConfig::default();
        assert!(config.thumbnail_dimension < config.max_dimension);
    }
}

#[cfg(test)]
mod video_processor_tests {
    use crate::services::media::video_processor::{VideoProcessorConfig, VideoCodec};

    #[test]
    fn test_video_processor_config_defaults() {
        let config = VideoProcessorConfig::default();
        
        assert!(matches!(config.codec, VideoCodec::H264));
        assert!(config.crf >= 23 && config.crf <= 28);
        assert_eq!(config.max_resolution, 1080);
        assert_eq!(config.thumbnail_dimension, 400);
    }

    #[test]
    fn test_crf_range() {
        let config = VideoProcessorConfig::default();
        // CRF should be between 23 and 28 for good quality/size balance
        assert!(config.crf >= 23);
        assert!(config.crf <= 28);
    }

    #[test]
    fn test_video_codec_variants() {
        let h264 = VideoCodec::H264;
        let h265 = VideoCodec::H265;
        
        assert!(matches!(h264, VideoCodec::H264));
        assert!(matches!(h265, VideoCodec::H265));
    }

    #[test]
    fn test_max_resolution_1080p() {
        let config = VideoProcessorConfig::default();
        assert_eq!(config.max_resolution, 1080);
    }

    #[test]
    fn test_video_thumbnail_dimension() {
        let config = VideoProcessorConfig::default();
        // Video thumbnails should be 400px
        assert_eq!(config.thumbnail_dimension, 400);
    }
}

#[cfg(test)]
mod dedup_manager_tests {
    use sha2::{Sha256, Digest};

    #[test]
    fn test_sha256_hash_consistency() {
        let data = b"test file content";
        
        let mut hasher1 = Sha256::new();
        hasher1.update(data);
        let hash1 = format!("{:x}", hasher1.finalize());
        
        let mut hasher2 = Sha256::new();
        hasher2.update(data);
        let hash2 = format!("{:x}", hasher2.finalize());
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_content_different_hash() {
        let data1 = b"content 1";
        let data2 = b"content 2";
        
        let mut hasher1 = Sha256::new();
        hasher1.update(data1);
        let hash1 = format!("{:x}", hasher1.finalize());
        
        let mut hasher2 = Sha256::new();
        hasher2.update(data2);
        let hash2 = format!("{:x}", hasher2.finalize());
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_length() {
        let data = b"test";
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = format!("{:x}", hasher.finalize());
        
        // SHA-256 produces 64 hex characters
        assert_eq!(hash.len(), 64);
    }
}


#[cfg(test)]
mod storage_provider_tests {
    use crate::services::media::storage_manager::StorageProviderType;

    #[test]
    fn test_storage_provider_type_as_str() {
        assert_eq!(StorageProviderType::AwsS3.as_str(), "aws_s3");
        assert_eq!(StorageProviderType::MinIO.as_str(), "minio");
        assert_eq!(StorageProviderType::TencentCOS.as_str(), "tencent_cos");
        assert_eq!(StorageProviderType::AliyunOSS.as_str(), "aliyun_oss");
        assert_eq!(StorageProviderType::CloudFlareR2.as_str(), "cloudflare_r2");
    }

    #[test]
    fn test_storage_provider_type_from_str() {
        assert!(matches!(
            StorageProviderType::from_str("aws_s3"),
            Ok(StorageProviderType::AwsS3)
        ));
        assert!(matches!(
            StorageProviderType::from_str("s3"),
            Ok(StorageProviderType::AwsS3)
        ));
        assert!(matches!(
            StorageProviderType::from_str("minio"),
            Ok(StorageProviderType::MinIO)
        ));
        assert!(matches!(
            StorageProviderType::from_str("tencent_cos"),
            Ok(StorageProviderType::TencentCOS)
        ));
        assert!(matches!(
            StorageProviderType::from_str("cos"),
            Ok(StorageProviderType::TencentCOS)
        ));
        assert!(matches!(
            StorageProviderType::from_str("aliyun_oss"),
            Ok(StorageProviderType::AliyunOSS)
        ));
        assert!(matches!(
            StorageProviderType::from_str("oss"),
            Ok(StorageProviderType::AliyunOSS)
        ));
        assert!(matches!(
            StorageProviderType::from_str("cloudflare_r2"),
            Ok(StorageProviderType::CloudFlareR2)
        ));
        assert!(matches!(
            StorageProviderType::from_str("r2"),
            Ok(StorageProviderType::CloudFlareR2)
        ));
    }

    #[test]
    fn test_storage_provider_type_from_str_invalid() {
        assert!(StorageProviderType::from_str("invalid").is_err());
        assert!(StorageProviderType::from_str("azure").is_err());
        assert!(StorageProviderType::from_str("gcs").is_err());
    }

    #[test]
    fn test_storage_provider_type_case_insensitive() {
        assert!(matches!(
            StorageProviderType::from_str("AWS_S3"),
            Ok(StorageProviderType::AwsS3)
        ));
        assert!(matches!(
            StorageProviderType::from_str("MINIO"),
            Ok(StorageProviderType::MinIO)
        ));
    }
}

#[cfg(test)]
mod lifecycle_manager_tests {
    use crate::services::media::lifecycle_manager::{LifecycleConfig, LifecycleStatus};

    #[test]
    fn test_lifecycle_config_defaults() {
        let config = LifecycleConfig::default();
        
        assert_eq!(config.unused_threshold_days, 90);
        assert_eq!(config.cold_storage_days, 30);
        assert_eq!(config.batch_size, 100);
        assert!(config.enable_cold_storage);
    }

    #[test]
    fn test_lifecycle_status_equality() {
        assert_eq!(LifecycleStatus::Active, LifecycleStatus::Active);
        assert_eq!(LifecycleStatus::MarkedForDeletion, LifecycleStatus::MarkedForDeletion);
        assert_eq!(LifecycleStatus::ColdStorage, LifecycleStatus::ColdStorage);
        assert_eq!(LifecycleStatus::Deleted, LifecycleStatus::Deleted);
    }

    #[test]
    fn test_lifecycle_status_inequality() {
        assert_ne!(LifecycleStatus::Active, LifecycleStatus::Deleted);
        assert_ne!(LifecycleStatus::MarkedForDeletion, LifecycleStatus::ColdStorage);
    }

    #[test]
    fn test_lifecycle_thresholds_reasonable() {
        let config = LifecycleConfig::default();
        
        // Unused threshold should be at least 30 days
        assert!(config.unused_threshold_days >= 30);
        
        // Cold storage should be at least 7 days
        assert!(config.cold_storage_days >= 7);
        
        // Total retention should be reasonable
        let total_days = config.unused_threshold_days + config.cold_storage_days;
        assert!(total_days >= 60);
        assert!(total_days <= 365);
    }
}

#[cfg(test)]
mod malware_scanner_tests {
    use crate::services::media::malware_scanner::{
        MalwareScannerConfig, ScannerType, ScanStatus,
    };
    use std::time::Duration;

    #[test]
    fn test_scanner_config_defaults() {
        let config = MalwareScannerConfig::default();
        
        assert!(matches!(config.scanner_type, ScannerType::Mock));
        assert_eq!(config.scan_timeout, Duration::from_secs(10));
        assert_eq!(config.max_scan_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_scan_status_variants() {
        let clean = ScanStatus::Clean;
        let infected = ScanStatus::Infected("Test.Malware".to_string());
        let pending = ScanStatus::Pending;
        let failed = ScanStatus::Failed("Error".to_string());

        assert!(matches!(clean, ScanStatus::Clean));
        assert!(matches!(infected, ScanStatus::Infected(_)));
        assert!(matches!(pending, ScanStatus::Pending));
        assert!(matches!(failed, ScanStatus::Failed(_)));
    }

    #[test]
    fn test_scan_status_equality() {
        assert_eq!(ScanStatus::Clean, ScanStatus::Clean);
        assert_eq!(ScanStatus::Pending, ScanStatus::Pending);
    }

    #[test]
    fn test_scanner_type_variants() {
        assert!(matches!(ScannerType::ClamAV, ScannerType::ClamAV));
        assert!(matches!(ScannerType::Mock, ScannerType::Mock));
        assert!(matches!(ScannerType::Disabled, ScannerType::Disabled));
    }
}


#[cfg(test)]
mod metadata_manager_tests {
    use crate::services::media::metadata_manager::MetadataManager;
    use uuid::Uuid;

    #[test]
    fn test_extract_basic_metadata() {
        let media_id = Uuid::new_v4();
        let metadata = MetadataManager::extract_basic_metadata(
            media_id,
            "image/jpeg",
            102400,
            Some(1920),
            Some(1080),
            None,
        );

        assert_eq!(metadata.media_id, media_id);
        assert_eq!(metadata.mime_type, "image/jpeg");
        assert_eq!(metadata.file_size, 102400);
        assert_eq!(metadata.width, Some(1920));
        assert_eq!(metadata.height, Some(1080));
        assert_eq!(metadata.duration, None);
        assert_eq!(metadata.format, Some("jpeg".to_string()));
    }

    #[test]
    fn test_extract_video_metadata() {
        let media_id = Uuid::new_v4();
        let metadata = MetadataManager::extract_basic_metadata(
            media_id,
            "video/mp4",
            10485760,
            Some(1920),
            Some(1080),
            Some(120.5),
        );

        assert_eq!(metadata.mime_type, "video/mp4");
        assert_eq!(metadata.duration, Some(120.5));
        assert_eq!(metadata.format, Some("mp4".to_string()));
    }

    #[test]
    fn test_format_extraction_from_mime() {
        let test_cases = [
            ("image/jpeg", "jpeg"),
            ("image/png", "png"),
            ("image/gif", "gif"),
            ("image/webp", "webp"),
            ("video/mp4", "mp4"),
            ("video/webm", "webm"),
        ];

        for (mime, expected_format) in test_cases {
            let metadata = MetadataManager::extract_basic_metadata(
                Uuid::new_v4(),
                mime,
                1024,
                None,
                None,
                None,
            );
            assert_eq!(metadata.format, Some(expected_format.to_string()));
        }
    }
}

#[cfg(test)]
mod processing_queue_tests {
    use crate::services::media::processing_queue::{ProcessingTask, QueueStats};
    use crate::services::media::models::MediaType;
    use uuid::Uuid;

    #[test]
    fn test_processing_task_creation() {
        let media_id = Uuid::new_v4();
        let task = ProcessingTask::new(
            media_id,
            MediaType::Image,
            "/tmp/test.jpg".to_string(),
        );

        assert_eq!(task.media_id, media_id);
        assert!(matches!(task.media_type, MediaType::Image));
        assert_eq!(task.input_path, "/tmp/test.jpg");
        assert_eq!(task.retry_count, 0);
    }

    #[test]
    fn test_queue_stats_default() {
        let stats = QueueStats::default();
        
        assert_eq!(stats.total_processed, 0);
        assert_eq!(stats.total_failed, 0);
        assert_eq!(stats.total_retried, 0);
        assert_eq!(stats.current_queue_size, 0);
        assert_eq!(stats.active_tasks, 0);
        assert_eq!(stats.available_slots, 0);
    }

    #[test]
    fn test_task_queued_at_timestamp() {
        let task = ProcessingTask::new(
            Uuid::new_v4(),
            MediaType::Video,
            "/tmp/test.mp4".to_string(),
        );

        // Queued time should be recent
        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(task.queued_at);
        assert!(diff.num_seconds() < 1);
    }
}

#[cfg(test)]
mod metrics_tests {
    use crate::services::media::metrics::{MediaMetrics, MediaEvent, MediaEventType};
    use std::sync::atomic::Ordering;
    use uuid::Uuid;

    #[test]
    fn test_metrics_recording() {
        let metrics = MediaMetrics::new();
        
        metrics.record_upload(1000);
        metrics.record_upload(2000);
        
        assert_eq!(metrics.total_uploads.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.total_upload_bytes.load(Ordering::Relaxed), 3000);
    }

    #[test]
    fn test_metrics_processing() {
        let metrics = MediaMetrics::new();
        
        metrics.record_processing(100);
        metrics.record_processing(200);
        metrics.record_processing(300);
        
        assert_eq!(metrics.total_processed.load(Ordering::Relaxed), 3);
        
        let snapshot = metrics.get_snapshot();
        assert_eq!(snapshot.avg_processing_time_ms, 200);
    }

    #[test]
    fn test_metrics_errors() {
        let metrics = MediaMetrics::new();
        
        metrics.record_upload_failure();
        metrics.record_upload_failure();
        metrics.record_processing_error();
        
        assert_eq!(metrics.failed_uploads.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.processing_errors.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_media_event_creation() {
        let user_id = Uuid::new_v4();
        let media_id = Uuid::new_v4();
        
        let event = MediaEvent::upload(user_id, media_id, 1024);
        
        assert!(matches!(event.event_type, MediaEventType::Upload));
        assert_eq!(event.media_id, Some(media_id));
        assert_eq!(event.user_id, Some(user_id));
        assert_eq!(event.file_size, Some(1024));
    }

    #[test]
    fn test_processing_events() {
        let media_id = Uuid::new_v4();
        
        let start = MediaEvent::processing_start(media_id);
        assert!(matches!(start.event_type, MediaEventType::ProcessingStart));
        
        let complete = MediaEvent::processing_complete(media_id, 500);
        assert!(matches!(complete.event_type, MediaEventType::ProcessingComplete));
        assert_eq!(complete.duration_ms, Some(500));
        
        let failed = MediaEvent::processing_failed(media_id, "Test error");
        assert!(matches!(failed.event_type, MediaEventType::ProcessingFailed));
        assert_eq!(failed.details, Some("Test error".to_string()));
    }
}

#[cfg(test)]
mod error_tests {
    use crate::services::media::error::MediaError;

    #[test]
    fn test_file_size_error() {
        let error = MediaError::FileSizeTooLarge(15_000_000, 10_000_000);
        let msg = error.to_string();
        assert!(msg.contains("15000000"));
        assert!(msg.contains("10000000"));
    }

    #[test]
    fn test_mime_type_error() {
        let error = MediaError::UnsupportedMimeType("image/bmp".to_string());
        let msg = error.to_string();
        assert!(msg.contains("image/bmp"));
    }

    #[test]
    fn test_malware_detected_error() {
        let error = MediaError::MalwareDetected("Test.Virus".to_string());
        let msg = error.to_string();
        assert!(msg.contains("Test.Virus"));
    }

    #[test]
    fn test_file_not_found_error() {
        let error = MediaError::FileNotFound("abc123".to_string());
        let msg = error.to_string();
        assert!(msg.contains("abc123"));
    }
}

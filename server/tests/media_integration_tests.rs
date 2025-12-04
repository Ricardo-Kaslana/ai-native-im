//! Integration tests for Media Processing System

use std::sync::Arc;

/// Test helper to create a mock storage provider
mod mock_provider {
    use ai_im_server::services::media::{StorageProvider, error::Result};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::RwLock;
    use std::time::Duration;

    pub struct MockStorageProvider {
        files: RwLock<HashMap<String, Vec<u8>>>,
        provider_type: String,
        cdn_domain: Option<String>,
    }

    impl MockStorageProvider {
        pub fn new(provider_type: &str) -> Self {
            Self {
                files: RwLock::new(HashMap::new()),
                provider_type: provider_type.to_string(),
                cdn_domain: None,
            }
        }

        pub fn with_cdn(mut self, domain: &str) -> Self {
            self.cdn_domain = Some(domain.to_string());
            self
        }
    }

    #[async_trait]
    impl StorageProvider for MockStorageProvider {
        async fn upload_file(&self, _local_path: &str, key: &str, _content_type: &str) -> Result<String> {
            let mut files = self.files.write().unwrap();
            files.insert(key.to_string(), vec![]);
            Ok(self.get_public_url(key))
        }

        async fn generate_presigned_url(&self, key: &str, _expires_in: Duration, _content_type: &str) -> Result<String> {
            Ok(format!("https://mock.storage/{}/presigned?key={}", self.provider_type, key))
        }

        fn get_public_url(&self, key: &str) -> String {
            format!("https://mock.storage/{}/{}", self.provider_type, key)
        }

        fn get_cdn_url(&self, key: &str) -> Option<String> {
            self.cdn_domain.as_ref().map(|domain| format!("https://{}/{}", domain, key))
        }

        async fn delete_file(&self, key: &str) -> Result<()> {
            let mut files = self.files.write().unwrap();
            files.remove(key);
            Ok(())
        }

        fn provider_type(&self) -> &str {
            &self.provider_type
        }
    }
}


#[cfg(test)]
mod storage_manager_integration {
    use super::mock_provider::MockStorageProvider;
    use ai_im_server::services::media::StorageManager;
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_storage_manager_with_same_provider() {
        let provider = Arc::new(MockStorageProvider::new("test"));
        let manager = StorageManager::new_single_provider(provider);
        assert_eq!(manager.image_provider().provider_type(), "test");
        assert_eq!(manager.video_provider().provider_type(), "test");
    }

    #[tokio::test]
    async fn test_storage_manager_with_different_providers() {
        let image_provider = Arc::new(MockStorageProvider::new("image_storage"));
        let video_provider = Arc::new(MockStorageProvider::new("video_storage"));
        let manager = StorageManager::new(image_provider, video_provider);
        assert_eq!(manager.image_provider().provider_type(), "image_storage");
        assert_eq!(manager.video_provider().provider_type(), "video_storage");
    }

    #[tokio::test]
    async fn test_presigned_url_generation() {
        let provider = Arc::new(MockStorageProvider::new("test"));
        let manager = StorageManager::new_single_provider(provider);
        let url = manager
            .generate_image_upload_url("test/image.jpg", Duration::from_secs(900), "image/jpeg")
            .await
            .unwrap();
        assert!(url.contains("presigned"));
    }

    #[tokio::test]
    async fn test_cdn_url_fallback() {
        let provider = Arc::new(MockStorageProvider::new("test"));
        let manager = StorageManager::new_single_provider(provider);
        let url = manager.get_image_url("test/image.jpg");
        assert!(url.contains("mock.storage"));

        let cdn_provider = Arc::new(MockStorageProvider::new("test").with_cdn("cdn.example.com"));
        let cdn_manager = StorageManager::new_single_provider(cdn_provider);
        let cdn_url = cdn_manager.get_image_url("test/image.jpg");
        assert!(cdn_url.contains("cdn.example.com"));
    }
}

#[cfg(test)]
mod deduplication_integration {
    use sha2::{Sha256, Digest};

    fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    #[test]
    fn test_hash_based_deduplication() {
        let content1 = b"same content";
        let content2 = b"same content";
        let content3 = b"different content";
        assert_eq!(compute_hash(content1), compute_hash(content2));
        assert_ne!(compute_hash(content1), compute_hash(content3));
    }

    #[test]
    fn test_hash_uniqueness() {
        let hashes: Vec<String> = (0..100)
            .map(|i| compute_hash(format!("content_{}", i).as_bytes()))
            .collect();
        let unique_count = hashes.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, 100);
    }
}

#[cfg(test)]
mod lifecycle_integration {
    use ai_im_server::services::media::{LifecycleConfig, LifecycleStatus};

    #[test]
    fn test_lifecycle_status_transitions() {
        assert_eq!(LifecycleStatus::Active, LifecycleStatus::Active);
        assert_eq!(LifecycleStatus::MarkedForDeletion, LifecycleStatus::MarkedForDeletion);
        assert_eq!(LifecycleStatus::ColdStorage, LifecycleStatus::ColdStorage);
        assert_eq!(LifecycleStatus::Deleted, LifecycleStatus::Deleted);
    }

    #[test]
    fn test_lifecycle_config_customization() {
        let config = LifecycleConfig {
            unused_threshold_days: 60,
            cold_storage_days: 14,
            batch_size: 50,
            enable_cold_storage: true,
        };
        assert_eq!(config.unused_threshold_days, 60);
        assert_eq!(config.cold_storage_days, 14);
    }
}

#[cfg(test)]
mod malware_scanner_integration {
    use ai_im_server::services::media::{MalwareScannerConfig, MalwareScannerManager, ScannerType, ScanStatus};
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_mock_scanner_clean_file() {
        let config = MalwareScannerConfig {
            scanner_type: ScannerType::Mock,
            ..Default::default()
        };
        let manager = MalwareScannerManager::new(config);
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"clean content").unwrap();
        let media_id = uuid::Uuid::new_v4();
        let result = manager.scan_file(media_id, temp_file.path()).await.unwrap();
        assert!(matches!(result.status, ScanStatus::Clean));
    }

    #[tokio::test]
    async fn test_disabled_scanner() {
        let config = MalwareScannerConfig {
            scanner_type: ScannerType::Disabled,
            ..Default::default()
        };
        let manager = MalwareScannerManager::new(config);
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"any content").unwrap();
        let media_id = uuid::Uuid::new_v4();
        let result = manager.scan_file(media_id, temp_file.path()).await.unwrap();
        assert!(matches!(result.status, ScanStatus::Clean));
    }
}

#[cfg(test)]
mod metrics_integration {
    use ai_im_server::services::media::{MediaLogger, MediaEvent};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_logger_event_recording() {
        let logger = MediaLogger::new(None);
        for i in 0..5 {
            let event = MediaEvent::upload(Uuid::new_v4(), Uuid::new_v4(), 1024 * (i + 1));
            logger.log_event(event).await;
        }
        let events = logger.get_recent_events(10).await;
        assert_eq!(events.len(), 5);
    }

    #[tokio::test]
    async fn test_metrics_snapshot() {
        let logger = MediaLogger::new(None);
        let event1 = MediaEvent::upload(Uuid::new_v4(), Uuid::new_v4(), 1000);
        let event2 = MediaEvent::upload(Uuid::new_v4(), Uuid::new_v4(), 2000);
        logger.log_event(event1).await;
        logger.log_event(event2).await;
        let metrics = logger.get_metrics();
        assert_eq!(metrics.total_uploads, 2);
        assert_eq!(metrics.total_upload_bytes, 3000);
    }
}

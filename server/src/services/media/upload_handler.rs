use super::config::MediaConfig;
use super::error::{MediaError, Result};
use super::malware_scanner::{MalwareScannerManager, ScanStatus};
use super::models::{MediaType, UploadToken};
use super::storage_manager::StorageManager;
use chrono::Utc;
use sqlx::PgPool;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Upload Handler - manages file upload requests and presigned URLs
pub struct UploadHandler {
    storage_manager: Arc<StorageManager>,
    malware_scanner: Arc<MalwareScannerManager>,
    config: MediaConfig,
    db_pool: PgPool,
}

impl UploadHandler {
    pub fn new(
        storage_manager: Arc<StorageManager>,
        malware_scanner: Arc<MalwareScannerManager>,
        config: MediaConfig,
        db_pool: PgPool,
    ) -> Self {
        Self {
            storage_manager,
            malware_scanner,
            config,
            db_pool,
        }
    }

    /// Request image upload - generates presigned URL
    pub async fn request_image_upload(
        &self,
        user_id: Uuid,
        file_size: u64,
        mime_type: String,
    ) -> Result<UploadToken> {
        // Validate file size
        if file_size > self.config.max_image_size {
            return Err(MediaError::FileSizeTooLarge(
                file_size,
                self.config.max_image_size,
            ));
        }

        // Validate MIME type
        if !matches!(
            mime_type.as_str(),
            "image/jpeg" | "image/png" | "image/gif"
        ) {
            return Err(MediaError::UnsupportedMimeType(mime_type.clone()));
        }

        self.create_upload_token(
            user_id,
            file_size,
            mime_type,
            MediaType::Image,
            self.config.image_upload_expiry,
        )
        .await
    }

    /// Request video upload - generates presigned URL
    pub async fn request_video_upload(
        &self,
        user_id: Uuid,
        file_size: u64,
        mime_type: String,
    ) -> Result<UploadToken> {
        // Validate file size
        if file_size > self.config.max_video_size {
            return Err(MediaError::FileSizeTooLarge(
                file_size,
                self.config.max_video_size,
            ));
        }

        // Validate MIME type
        if !matches!(
            mime_type.as_str(),
            "video/mp4" | "video/quicktime" | "video/webm"
        ) {
            return Err(MediaError::UnsupportedMimeType(mime_type.clone()));
        }

        self.create_upload_token(
            user_id,
            file_size,
            mime_type,
            MediaType::Video,
            self.config.video_upload_expiry,
        )
        .await
    }

    /// Create upload token and presigned URL
    async fn create_upload_token(
        &self,
        user_id: Uuid,
        file_size: u64,
        mime_type: String,
        media_type: MediaType,
        expiry_secs: u64,
    ) -> Result<UploadToken> {
        // Generate media ID
        let media_id = Uuid::new_v4();
        
        // Generate storage key
        let extension = self.get_extension(&mime_type);
        let storage_key = format!(
            "uploads/{}/{}/{}{}",
            match media_type {
                MediaType::Image => "images",
                MediaType::Video => "videos",
            },
            user_id,
            media_id,
            extension
        );

        // Generate presigned URL using appropriate provider
        let upload_url = match media_type {
            MediaType::Image => {
                self.storage_manager
                    .generate_image_upload_url(
                        &storage_key,
                        Duration::from_secs(expiry_secs),
                        &mime_type,
                    )
                    .await?
            }
            MediaType::Video => {
                self.storage_manager
                    .generate_video_upload_url(
                        &storage_key,
                        Duration::from_secs(expiry_secs),
                        &mime_type,
                    )
                    .await?
            }
        };

        // Get the public URL for the file
        let original_url = match media_type {
            MediaType::Image => self.storage_manager.get_image_url(&storage_key),
            MediaType::Video => self.storage_manager.get_video_url(&storage_key),
        };

        // Get the storage provider type
        let storage_provider = match media_type {
            MediaType::Image => self.storage_manager.image_provider().provider_type(),
            MediaType::Video => self.storage_manager.video_provider().provider_type(),
        };

        // Create media record in database with placeholder hash
        let placeholder_hash = format!("pending_{}", media_id);

        sqlx::query!(
            r#"
            INSERT INTO media_files (
                id, uploader_id, file_hash, media_type, mime_type,
                storage_provider, original_url, file_size, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            media_id,
            user_id,
            placeholder_hash,
            match media_type {
                MediaType::Image => "image",
                MediaType::Video => "video",
            },
            mime_type,
            storage_provider,
            original_url,
            file_size as i64,
            "processing"
        )
        .execute(&self.db_pool)
        .await?;

        // Generate token (simple UUID for now)
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now().timestamp() + expiry_secs as i64;

        Ok(UploadToken {
            token,
            upload_url,
            expires_at,
            media_id,
        })
    }

    /// Complete upload - update file hash, scan for malware, and trigger processing
    pub async fn complete_upload(&self, media_id: Uuid, file_hash: String) -> Result<Uuid> {
        // Update file hash in database
        sqlx::query!(
            r#"
            UPDATE media_files
            SET file_hash = $1, updated_at = NOW()
            WHERE id = $2
            "#,
            file_hash,
            media_id
        )
        .execute(&self.db_pool)
        .await?;

        // TODO: Queue processing task
        // For now, just return the media ID

        Ok(media_id)
    }

    /// Scan uploaded file for malware
    /// Returns Ok(()) if clean or pending, Err if malware detected
    pub async fn scan_uploaded_file(
        &self,
        media_id: Uuid,
        file_path: &Path,
    ) -> Result<ScanStatus> {
        info!("Scanning file for malware: {:?}", file_path);
        
        let result = self.malware_scanner.scan_file(media_id, file_path).await?;
        
        match &result.status {
            ScanStatus::Clean => {
                info!(
                    "File scan clean: {:?} ({}ms)",
                    file_path, result.scan_duration_ms
                );
                // Update scan status in database
                self.update_scan_status(media_id, "clean", None).await?;
                Ok(ScanStatus::Clean)
            }
            ScanStatus::Infected(malware_name) => {
                error!(
                    "Malware detected in file {:?}: {}",
                    file_path, malware_name
                );
                // Update scan status and mark file as infected
                self.update_scan_status(media_id, "infected", Some(malware_name))
                    .await?;
                // Delete the infected file
                self.delete_infected_file(media_id).await?;
                Err(MediaError::MalwareDetected(malware_name.clone()))
            }
            ScanStatus::Pending => {
                warn!("File scan pending (scanner unavailable): {:?}", file_path);
                // Update scan status to pending
                self.update_scan_status(media_id, "pending", None).await?;
                Ok(ScanStatus::Pending)
            }
            ScanStatus::Failed(reason) => {
                error!("File scan failed: {:?} - {}", file_path, reason);
                // Update scan status to failed but allow upload
                self.update_scan_status(media_id, "failed", Some(reason)).await?;
                Ok(ScanStatus::Failed(reason.clone()))
            }
        }
    }

    /// Update scan status in database
    async fn update_scan_status(
        &self,
        media_id: Uuid,
        status: &str,
        details: Option<&str>,
    ) -> Result<()> {
        // For now, we'll store scan status in the status field
        // In production, you might want a separate scan_status column
        if status == "infected" {
            sqlx::query!(
                r#"
                UPDATE media_files
                SET status = 'infected', updated_at = NOW()
                WHERE id = $1
                "#,
                media_id
            )
            .execute(&self.db_pool)
            .await?;
        }
        
        info!(
            "Updated scan status for media {}: {} {:?}",
            media_id, status, details
        );
        Ok(())
    }

    /// Delete infected file from storage and database
    async fn delete_infected_file(&self, media_id: Uuid) -> Result<()> {
        // Get file info
        let file = sqlx::query!(
            r#"
            SELECT storage_provider, original_url, media_type
            FROM media_files
            WHERE id = $1
            "#,
            media_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(file) = file {
            // Extract storage key from URL
            if let Some(key) = self.extract_storage_key(&file.original_url) {
                // Delete from storage
                let is_image = file.media_type == "image";
                if is_image {
                    let _ = self.storage_manager.image_provider().delete_file(&key).await;
                } else {
                    let _ = self.storage_manager.video_provider().delete_file(&key).await;
                }
            }
        }

        // Delete from database
        sqlx::query!(
            r#"
            DELETE FROM media_files
            WHERE id = $1
            "#,
            media_id
        )
        .execute(&self.db_pool)
        .await?;

        info!("Deleted infected file: {}", media_id);
        Ok(())
    }

    /// Extract storage key from URL
    fn extract_storage_key(&self, url: &str) -> Option<String> {
        // Simple extraction - in production, this would be more robust
        url.split('/').last().map(|s| s.to_string())
    }

    /// Process pending malware scans
    pub async fn process_pending_scans(&self) -> Vec<(Uuid, ScanStatus)> {
        let results = self.malware_scanner.process_pending_scans().await;
        let mut processed = Vec::new();

        for (media_id, result) in results {
            match &result.status {
                ScanStatus::Infected(malware_name) => {
                    error!("Pending scan found malware for {}: {}", media_id, malware_name);
                    if let Err(e) = self.delete_infected_file(media_id).await {
                        error!("Failed to delete infected file {}: {}", media_id, e);
                    }
                    // TODO: Notify affected users
                    processed.push((media_id, ScanStatus::Infected(malware_name.clone())));
                }
                ScanStatus::Clean => {
                    info!("Pending scan clean for {}", media_id);
                    let _ = self.update_scan_status(media_id, "clean", None).await;
                    processed.push((media_id, ScanStatus::Clean));
                }
                _ => {}
            }
        }

        processed
    }

    /// Get count of pending scans
    pub async fn pending_scan_count(&self) -> usize {
        self.malware_scanner.pending_count().await
    }

    /// Get file extension from MIME type
    fn get_extension(&self, mime_type: &str) -> &'static str {
        match mime_type {
            "image/jpeg" => ".jpg",
            "image/png" => ".png",
            "image/gif" => ".gif",
            "video/mp4" => ".mp4",
            "video/quicktime" => ".mov",
            "video/webm" => ".webm",
            _ => "",
        }
    }
}

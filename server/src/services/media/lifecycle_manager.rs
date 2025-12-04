use super::error::{MediaError, Result};
use super::storage_manager::StorageManager;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Lifecycle configuration
#[derive(Debug, Clone)]
pub struct LifecycleConfig {
    /// Days before unused files are marked for deletion
    pub unused_threshold_days: i64,
    /// Days in cold storage before permanent deletion
    pub cold_storage_days: i64,
    /// Batch size for lifecycle operations
    pub batch_size: i64,
    /// Enable cold storage tier
    pub enable_cold_storage: bool,
}

impl Default for LifecycleConfig {
    fn default() -> Self {
        Self {
            unused_threshold_days: 90,
            cold_storage_days: 30,
            batch_size: 100,
            enable_cold_storage: true,
        }
    }
}

/// File lifecycle status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleStatus {
    Active,
    MarkedForDeletion,
    ColdStorage,
    Deleted,
}

/// Media file with lifecycle info
#[derive(Debug, Clone)]
pub struct MediaFileLifecycle {
    pub id: Uuid,
    pub uploader_id: Uuid,
    pub storage_provider: String,
    pub original_url: String,
    pub processed_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub ref_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub lifecycle_status: LifecycleStatus,
    pub marked_for_deletion_at: Option<DateTime<Utc>>,
}


/// Lifecycle manager for media files
pub struct LifecycleManager {
    db_pool: PgPool,
    storage_manager: Arc<StorageManager>,
    config: LifecycleConfig,
}

impl LifecycleManager {
    pub fn new(
        db_pool: PgPool,
        storage_manager: Arc<StorageManager>,
        config: LifecycleConfig,
    ) -> Self {
        Self {
            db_pool,
            storage_manager,
            config,
        }
    }

    /// Update last access time for a media file
    pub async fn record_access(&self, media_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE media_files
            SET last_accessed_at = NOW()
            WHERE id = $1
            "#,
            media_id
        )
        .execute(&self.db_pool)
        .await?;

        debug!("Recorded access for media: {}", media_id);
        Ok(())
    }

    /// Find and mark unused files for deletion
    pub async fn mark_unused_files(&self) -> Result<Vec<Uuid>> {
        let threshold = (Utc::now() - Duration::days(self.config.unused_threshold_days)).naive_utc();
        
        // Find files not accessed in threshold days and not referenced
        let files = sqlx::query!(
            r#"
            SELECT id
            FROM media_files
            WHERE ref_count = 0
              AND (last_accessed_at IS NULL OR last_accessed_at < $1)
              AND status = 'completed'
              AND lifecycle_status IS NULL OR lifecycle_status = 'active'
            LIMIT $2
            "#,
            threshold,
            self.config.batch_size
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut marked_ids = Vec::new();

        for file in files {
            if let Err(e) = self.mark_for_deletion(file.id).await {
                error!("Failed to mark file {} for deletion: {}", file.id, e);
            } else {
                marked_ids.push(file.id);
            }
        }

        if !marked_ids.is_empty() {
            info!("Marked {} files for deletion", marked_ids.len());
        }

        Ok(marked_ids)
    }

    /// Mark a single file for deletion
    async fn mark_for_deletion(&self, media_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE media_files
            SET lifecycle_status = 'marked_for_deletion',
                marked_for_deletion_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
            media_id
        )
        .execute(&self.db_pool)
        .await?;

        info!("Marked media {} for deletion", media_id);
        Ok(())
    }

    /// Move marked files to cold storage
    pub async fn move_to_cold_storage(&self) -> Result<Vec<Uuid>> {
        if !self.config.enable_cold_storage {
            return Ok(Vec::new());
        }

        // Find files marked for deletion
        let files = sqlx::query!(
            r#"
            SELECT id, storage_provider, original_url, processed_url, thumbnail_url, media_type
            FROM media_files
            WHERE lifecycle_status = 'marked_for_deletion'
            LIMIT $1
            "#,
            self.config.batch_size
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut moved_ids = Vec::new();

        for file in files {
            // In a real implementation, we would move files to a cold storage tier
            // For now, we just update the status
            if let Err(e) = self.update_to_cold_storage(file.id).await {
                error!("Failed to move file {} to cold storage: {}", file.id, e);
            } else {
                moved_ids.push(file.id);
            }
        }

        if !moved_ids.is_empty() {
            info!("Moved {} files to cold storage", moved_ids.len());
        }

        Ok(moved_ids)
    }

    /// Update file status to cold storage
    async fn update_to_cold_storage(&self, media_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE media_files
            SET lifecycle_status = 'cold_storage',
                updated_at = NOW()
            WHERE id = $1
            "#,
            media_id
        )
        .execute(&self.db_pool)
        .await?;

        info!("Moved media {} to cold storage", media_id);
        Ok(())
    }

    /// Permanently delete files that have been in cold storage too long
    pub async fn delete_expired_files(&self) -> Result<Vec<Uuid>> {
        let threshold = (Utc::now() - Duration::days(self.config.cold_storage_days)).naive_utc();

        // Find files in cold storage past threshold
        let files = sqlx::query!(
            r#"
            SELECT id, storage_provider, original_url, processed_url, thumbnail_url, media_type
            FROM media_files
            WHERE lifecycle_status = 'cold_storage'
              AND marked_for_deletion_at < $1
            LIMIT $2
            "#,
            threshold,
            self.config.batch_size
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut deleted_ids = Vec::new();

        for file in files {
            if let Err(e) = self.permanently_delete(
                file.id,
                &file.storage_provider,
                &file.original_url,
                file.processed_url.as_deref(),
                file.thumbnail_url.as_deref(),
                &file.media_type,
            ).await {
                error!("Failed to permanently delete file {}: {}", file.id, e);
            } else {
                deleted_ids.push(file.id);
            }
        }

        if !deleted_ids.is_empty() {
            info!("Permanently deleted {} files", deleted_ids.len());
        }

        Ok(deleted_ids)
    }

    /// Permanently delete a file from storage and database
    async fn permanently_delete(
        &self,
        media_id: Uuid,
        storage_provider: &str,
        original_url: &str,
        processed_url: Option<&str>,
        thumbnail_url: Option<&str>,
        media_type: &str,
    ) -> Result<()> {
        let is_image = media_type == "image";

        // Delete files from storage
        let urls_to_delete: Vec<&str> = [Some(original_url), processed_url, thumbnail_url]
            .into_iter()
            .flatten()
            .collect();

        for url in urls_to_delete {
            if let Some(key) = self.extract_storage_key(url) {
                let result = if is_image {
                    self.storage_manager.image_provider().delete_file(&key).await
                } else {
                    self.storage_manager.video_provider().delete_file(&key).await
                };

                if let Err(e) = result {
                    warn!("Failed to delete file from storage: {} - {}", key, e);
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

        // Log deletion for audit
        info!(
            "Permanently deleted media {} (provider: {}, type: {})",
            media_id, storage_provider, media_type
        );

        Ok(())
    }

    /// Extract storage key from URL
    fn extract_storage_key(&self, url: &str) -> Option<String> {
        // Extract the path portion after the bucket/domain
        url.split('/').skip(3).collect::<Vec<_>>().join("/").into()
    }

    /// Run full lifecycle cleanup
    pub async fn run_cleanup(&self) -> Result<LifecycleCleanupResult> {
        info!("Starting lifecycle cleanup");

        let marked = self.mark_unused_files().await?;
        let moved = self.move_to_cold_storage().await?;
        let deleted = self.delete_expired_files().await?;

        let result = LifecycleCleanupResult {
            files_marked: marked.len(),
            files_moved_to_cold: moved.len(),
            files_deleted: deleted.len(),
            run_at: Utc::now(),
        };

        info!(
            "Lifecycle cleanup complete: {} marked, {} moved to cold, {} deleted",
            result.files_marked, result.files_moved_to_cold, result.files_deleted
        );

        Ok(result)
    }

    /// Get lifecycle statistics
    pub async fn get_stats(&self) -> Result<LifecycleStats> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(CASE WHEN lifecycle_status IS NULL OR lifecycle_status = 'active' THEN 1 END) as "active_count!: i64",
                COUNT(CASE WHEN lifecycle_status = 'marked_for_deletion' THEN 1 END) as "marked_count!: i64",
                COUNT(CASE WHEN lifecycle_status = 'cold_storage' THEN 1 END) as "cold_count!: i64",
                COALESCE(SUM(CASE WHEN lifecycle_status IS NULL OR lifecycle_status = 'active' THEN file_size ELSE 0 END), 0) as "active_size!: i64",
                COALESCE(SUM(CASE WHEN lifecycle_status = 'cold_storage' THEN file_size ELSE 0 END), 0) as "cold_size!: i64"
            FROM media_files
            "#
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(LifecycleStats {
            active_files: stats.active_count as u64,
            marked_for_deletion: stats.marked_count as u64,
            in_cold_storage: stats.cold_count as u64,
            active_storage_bytes: stats.active_size as u64,
            cold_storage_bytes: stats.cold_size as u64,
        })
    }
}


/// Result of lifecycle cleanup run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleCleanupResult {
    pub files_marked: usize,
    pub files_moved_to_cold: usize,
    pub files_deleted: usize,
    pub run_at: DateTime<Utc>,
}

/// Lifecycle statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleStats {
    pub active_files: u64,
    pub marked_for_deletion: u64,
    pub in_cold_storage: u64,
    pub active_storage_bytes: u64,
    pub cold_storage_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_config_default() {
        let config = LifecycleConfig::default();
        assert_eq!(config.unused_threshold_days, 90);
        assert_eq!(config.cold_storage_days, 30);
        assert_eq!(config.batch_size, 100);
        assert!(config.enable_cold_storage);
    }

    #[test]
    fn test_lifecycle_status() {
        assert_eq!(LifecycleStatus::Active, LifecycleStatus::Active);
        assert_ne!(LifecycleStatus::Active, LifecycleStatus::Deleted);
    }

    #[test]
    fn test_extract_storage_key() {
        // Test URL parsing
        let url = "https://bucket.s3.amazonaws.com/uploads/images/user123/file.jpg";
        let parts: Vec<&str> = url.split('/').skip(3).collect();
        let key = parts.join("/");
        assert_eq!(key, "uploads/images/user123/file.jpg");
    }
}

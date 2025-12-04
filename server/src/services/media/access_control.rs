use super::error::{MediaError, Result};
use super::storage_manager::StorageManager;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Default presigned URL expiration (1 hour)
const DEFAULT_URL_EXPIRY: Duration = Duration::from_secs(3600);

/// Access control manager for media files
pub struct AccessControlManager {
    db_pool: PgPool,
    storage_manager: Arc<StorageManager>,
}

impl AccessControlManager {
    pub fn new(db_pool: PgPool, storage_manager: Arc<StorageManager>) -> Self {
        Self {
            db_pool,
            storage_manager,
        }
    }

    /// Check if user is the owner of a media file
    pub async fn is_owner(&self, media_id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            SELECT uploader_id
            FROM media_files
            WHERE id = $1
            "#,
            media_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(result.map(|r| r.uploader_id == user_id).unwrap_or(false))
    }

    /// Check if a media file is referenced by any messages
    pub async fn is_referenced(&self, media_id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            SELECT ref_count
            FROM media_files
            WHERE id = $1
            "#,
            media_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(result.map(|r| r.ref_count > 0).unwrap_or(false))
    }

    /// Verify user can delete a media file
    pub async fn can_delete(&self, media_id: Uuid, user_id: Uuid) -> Result<DeletePermission> {
        // Check ownership
        if !self.is_owner(media_id, user_id).await? {
            return Ok(DeletePermission::Denied(
                "You are not the owner of this file".to_string(),
            ));
        }

        // Check references
        if self.is_referenced(media_id).await? {
            return Ok(DeletePermission::Denied(
                "File is still referenced by messages".to_string(),
            ));
        }

        Ok(DeletePermission::Allowed)
    }

    /// Delete a media file with access control
    pub async fn delete_media(&self, media_id: Uuid, user_id: Uuid) -> Result<()> {
        // Check permission
        match self.can_delete(media_id, user_id).await? {
            DeletePermission::Allowed => {}
            DeletePermission::Denied(reason) => {
                if reason.contains("owner") {
                    return Err(MediaError::Unauthorized);
                } else {
                    return Err(MediaError::FileStillReferenced);
                }
            }
        }

        // Get file info for deletion
        let file = sqlx::query!(
            r#"
            SELECT storage_provider, original_url, processed_url, thumbnail_url, media_type
            FROM media_files
            WHERE id = $1
            "#,
            media_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| MediaError::FileNotFound(media_id.to_string()))?;

        // Delete from storage
        let is_image = file.media_type == "image";
        let urls = [
            Some(file.original_url.as_str()),
            file.processed_url.as_deref(),
            file.thumbnail_url.as_deref(),
        ];

        for url in urls.into_iter().flatten() {
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

        // Delete metadata
        sqlx::query!(
            "DELETE FROM media_metadata WHERE media_id = $1",
            media_id
        )
        .execute(&self.db_pool)
        .await?;

        // Delete from database
        sqlx::query!(
            "DELETE FROM media_files WHERE id = $1",
            media_id
        )
        .execute(&self.db_pool)
        .await?;

        info!("Deleted media file: {} by user: {}", media_id, user_id);
        Ok(())
    }


    /// Generate presigned URL for private access
    pub async fn generate_private_url(
        &self,
        media_id: Uuid,
        user_id: Uuid,
        expiry: Option<Duration>,
    ) -> Result<String> {
        // Verify user has access (owner or referenced in their messages)
        let has_access = self.check_access(media_id, user_id).await?;
        if !has_access {
            return Err(MediaError::Unauthorized);
        }

        // Get file info
        let file = sqlx::query!(
            r#"
            SELECT storage_provider, original_url, processed_url, media_type
            FROM media_files
            WHERE id = $1
            "#,
            media_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| MediaError::FileNotFound(media_id.to_string()))?;

        // Use processed URL if available, otherwise original
        let url = file.processed_url.as_ref().unwrap_or(&file.original_url);
        let key = self
            .extract_storage_key(url)
            .ok_or_else(|| MediaError::InternalError("Invalid storage URL".to_string()))?;

        let expiry = expiry.unwrap_or(DEFAULT_URL_EXPIRY);
        let is_image = file.media_type == "image";

        // Determine content type based on media type
        let content_type = if is_image { "image/webp" } else { "video/mp4" };
        
        let presigned_url = if is_image {
            self.storage_manager
                .image_provider()
                .generate_presigned_url(&key, expiry, content_type)
                .await?
        } else {
            self.storage_manager
                .video_provider()
                .generate_presigned_url(&key, expiry, content_type)
                .await?
        };

        debug!(
            "Generated presigned URL for media {} (expires in {:?})",
            media_id, expiry
        );

        Ok(presigned_url)
    }

    /// Check if user has access to a media file
    async fn check_access(&self, media_id: Uuid, user_id: Uuid) -> Result<bool> {
        // Owner always has access
        if self.is_owner(media_id, user_id).await? {
            return Ok(true);
        }

        // Check if user has received a message with this media
        // This would require a messages table join - simplified for now
        // In production, implement proper message-media relationship check
        
        Ok(false)
    }

    /// Extract storage key from URL
    fn extract_storage_key(&self, url: &str) -> Option<String> {
        // Extract path after domain/bucket
        url.split('/').skip(3).collect::<Vec<_>>().join("/").into()
    }

    /// Increment reference count when media is attached to a message
    pub async fn add_reference(&self, media_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE media_files
            SET ref_count = COALESCE(ref_count, 0) + 1,
                updated_at = NOW()
            WHERE id = $1
            "#,
            media_id
        )
        .execute(&self.db_pool)
        .await?;

        debug!("Added reference to media: {}", media_id);
        Ok(())
    }

    /// Decrement reference count when message is deleted
    pub async fn remove_reference(&self, media_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE media_files
            SET ref_count = GREATEST(COALESCE(ref_count, 1) - 1, 0),
                updated_at = NOW()
            WHERE id = $1
            "#,
            media_id
        )
        .execute(&self.db_pool)
        .await?;

        debug!("Removed reference from media: {}", media_id);
        Ok(())
    }

    /// Get media file info for access check
    pub async fn get_media_owner(&self, media_id: Uuid) -> Result<Option<Uuid>> {
        let result = sqlx::query!(
            "SELECT uploader_id FROM media_files WHERE id = $1",
            media_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(result.map(|r| r.uploader_id))
    }
}

/// Delete permission result
#[derive(Debug, Clone)]
pub enum DeletePermission {
    Allowed,
    Denied(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_permission() {
        let allowed = DeletePermission::Allowed;
        let denied = DeletePermission::Denied("test reason".to_string());

        assert!(matches!(allowed, DeletePermission::Allowed));
        assert!(matches!(denied, DeletePermission::Denied(_)));
    }

    #[test]
    fn test_default_url_expiry() {
        assert_eq!(DEFAULT_URL_EXPIRY, Duration::from_secs(3600));
    }

    #[test]
    fn test_extract_storage_key() {
        let url = "https://bucket.s3.amazonaws.com/uploads/images/user/file.jpg";
        let parts: Vec<&str> = url.split('/').skip(3).collect();
        let key = parts.join("/");
        assert_eq!(key, "uploads/images/user/file.jpg");
    }
}

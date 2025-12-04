use super::error::{MediaError, Result};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Cache TTL for metadata (1 hour)
const METADATA_CACHE_TTL: u64 = 3600;

/// Redis key prefix for metadata cache
const CACHE_PREFIX: &str = "media:metadata:";

/// Media metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub media_id: Uuid,
    pub format: Option<String>,
    pub codec: Option<String>,
    pub bitrate: Option<i32>,
    pub framerate: Option<f32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<f32>,
    pub file_size: i64,
    pub mime_type: String,
    pub extra: Option<serde_json::Value>,
}

/// Full media info including URLs and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub id: Uuid,
    pub media_type: String,
    pub mime_type: String,
    pub original_url: String,
    pub processed_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub cdn_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<f32>,
    pub file_size: i64,
    pub status: String,
    pub metadata: Option<MediaMetadata>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}


/// Metadata manager for media files
pub struct MetadataManager {
    db_pool: PgPool,
    redis_client: Option<redis::Client>,
}

impl MetadataManager {
    pub fn new(db_pool: PgPool, redis_url: Option<&str>) -> Self {
        let redis_client = redis_url.and_then(|url| {
            redis::Client::open(url)
                .map_err(|e| {
                    warn!("Failed to connect to Redis for metadata cache: {}", e);
                    e
                })
                .ok()
        });

        Self {
            db_pool,
            redis_client,
        }
    }

    /// Store metadata for a media file
    pub async fn store_metadata(&self, metadata: &MediaMetadata) -> Result<()> {
        // Store in database
        sqlx::query!(
            r#"
            INSERT INTO media_metadata (media_id, format, codec, bitrate, framerate, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (media_id) DO UPDATE SET
                format = EXCLUDED.format,
                codec = EXCLUDED.codec,
                bitrate = EXCLUDED.bitrate,
                framerate = EXCLUDED.framerate,
                metadata = EXCLUDED.metadata
            "#,
            metadata.media_id,
            metadata.format,
            metadata.codec,
            metadata.bitrate,
            metadata.framerate,
            metadata.extra
        )
        .execute(&self.db_pool)
        .await?;

        // Invalidate cache
        self.invalidate_cache(metadata.media_id).await;

        debug!("Stored metadata for media: {}", metadata.media_id);
        Ok(())
    }

    /// Get metadata for a media file (with caching)
    pub async fn get_metadata(&self, media_id: Uuid) -> Result<Option<MediaMetadata>> {
        // Try cache first
        if let Some(cached) = self.get_from_cache(media_id).await {
            debug!("Cache hit for metadata: {}", media_id);
            return Ok(Some(cached));
        }

        // Fetch from database
        let result = sqlx::query!(
            r#"
            SELECT 
                m.id, m.mime_type, m.file_size, m.width, m.height, m.duration,
                mm.format, mm.codec, mm.bitrate, mm.framerate, mm.metadata as extra
            FROM media_files m
            LEFT JOIN media_metadata mm ON m.id = mm.media_id
            WHERE m.id = $1
            "#,
            media_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(row) = result {
            let metadata = MediaMetadata {
                media_id,
                format: row.format,
                codec: row.codec,
                bitrate: row.bitrate,
                framerate: row.framerate,
                width: row.width,
                height: row.height,
                duration: row.duration,
                file_size: row.file_size,
                mime_type: row.mime_type,
                extra: row.extra,
            };

            // Cache the result
            self.set_cache(media_id, &metadata).await;

            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    /// Get full media info including URLs
    pub async fn get_media_info(&self, media_id: Uuid) -> Result<Option<MediaInfo>> {
        let result = sqlx::query!(
            r#"
            SELECT 
                m.id, m.media_type, m.mime_type, m.original_url, m.processed_url,
                m.thumbnail_url, m.width, m.height, m.duration, m.file_size,
                m.status, m.created_at,
                mm.format, mm.codec, mm.bitrate, mm.framerate, mm.metadata as extra
            FROM media_files m
            LEFT JOIN media_metadata mm ON m.id = mm.media_id
            WHERE m.id = $1
            "#,
            media_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(row) = result {
            let metadata = if row.format.is_some() || row.codec.is_some() {
                Some(MediaMetadata {
                    media_id,
                    format: row.format,
                    codec: row.codec,
                    bitrate: row.bitrate,
                    framerate: row.framerate,
                    width: row.width,
                    height: row.height,
                    duration: row.duration,
                    file_size: row.file_size,
                    mime_type: row.mime_type.clone(),
                    extra: row.extra,
                })
            } else {
                None
            };

            Ok(Some(MediaInfo {
                id: row.id,
                media_type: row.media_type,
                mime_type: row.mime_type,
                original_url: row.original_url,
                processed_url: row.processed_url,
                thumbnail_url: row.thumbnail_url,
                cdn_url: None, // Will be populated by storage manager
                width: row.width,
                height: row.height,
                duration: row.duration,
                file_size: row.file_size,
                status: row.status,
                metadata,
                created_at: row.created_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Update metadata
    pub async fn update_metadata(&self, media_id: Uuid, updates: MetadataUpdate) -> Result<()> {
        if let Some(format) = &updates.format {
            sqlx::query!(
                "UPDATE media_metadata SET format = $1 WHERE media_id = $2",
                format,
                media_id
            )
            .execute(&self.db_pool)
            .await?;
        }

        if let Some(codec) = &updates.codec {
            sqlx::query!(
                "UPDATE media_metadata SET codec = $1 WHERE media_id = $2",
                codec,
                media_id
            )
            .execute(&self.db_pool)
            .await?;
        }

        if let Some(extra) = &updates.extra {
            sqlx::query!(
                "UPDATE media_metadata SET metadata = $1 WHERE media_id = $2",
                extra,
                media_id
            )
            .execute(&self.db_pool)
            .await?;
        }

        // Invalidate cache
        self.invalidate_cache(media_id).await;

        info!("Updated metadata for media: {}", media_id);
        Ok(())
    }

    /// Invalidate cache for a media file
    pub async fn invalidate_cache(&self, media_id: Uuid) {
        if let Some(ref client) = self.redis_client {
            if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                let key = format!("{}{}", CACHE_PREFIX, media_id);
                let _: std::result::Result<(), redis::RedisError> = conn.del(&key).await;
                debug!("Invalidated cache for media: {}", media_id);
            }
        }
    }

    /// Get metadata from cache
    async fn get_from_cache(&self, media_id: Uuid) -> Option<MediaMetadata> {
        let client = self.redis_client.as_ref()?;
        let mut conn = client.get_multiplexed_async_connection().await.ok()?;
        
        let key = format!("{}{}", CACHE_PREFIX, media_id);
        let data: Option<String> = conn.get(&key).await.ok()?;
        
        data.and_then(|s| serde_json::from_str(&s).ok())
    }

    /// Set metadata in cache
    async fn set_cache(&self, media_id: Uuid, metadata: &MediaMetadata) {
        if let Some(ref client) = self.redis_client {
            if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                let key = format!("{}{}", CACHE_PREFIX, media_id);
                if let Ok(json) = serde_json::to_string(metadata) {
                    let _: std::result::Result<(), redis::RedisError> = conn
                        .set_ex(&key, json, METADATA_CACHE_TTL)
                        .await;
                    debug!("Cached metadata for media: {}", media_id);
                }
            }
        }
    }

    /// Extract basic metadata from file info
    pub fn extract_basic_metadata(
        media_id: Uuid,
        mime_type: &str,
        file_size: i64,
        width: Option<i32>,
        height: Option<i32>,
        duration: Option<f32>,
    ) -> MediaMetadata {
        let format = mime_type.split('/').last().map(|s| s.to_string());
        
        MediaMetadata {
            media_id,
            format,
            codec: None,
            bitrate: None,
            framerate: None,
            width,
            height,
            duration,
            file_size,
            mime_type: mime_type.to_string(),
            extra: None,
        }
    }
}

/// Metadata update request
#[derive(Debug, Clone, Default)]
pub struct MetadataUpdate {
    pub format: Option<String>,
    pub codec: Option<String>,
    pub bitrate: Option<i32>,
    pub framerate: Option<f32>,
    pub extra: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_format() {
        let media_id = Uuid::new_v4();
        let key = format!("{}{}", CACHE_PREFIX, media_id);
        assert!(key.starts_with("media:metadata:"));
    }

    #[test]
    fn test_extract_basic_metadata() {
        let media_id = Uuid::new_v4();
        let metadata = MetadataManager::extract_basic_metadata(
            media_id,
            "image/jpeg",
            1024,
            Some(800),
            Some(600),
            None,
        );

        assert_eq!(metadata.media_id, media_id);
        assert_eq!(metadata.format, Some("jpeg".to_string()));
        assert_eq!(metadata.file_size, 1024);
        assert_eq!(metadata.width, Some(800));
        assert_eq!(metadata.height, Some(600));
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = MediaMetadata {
            media_id: Uuid::new_v4(),
            format: Some("webp".to_string()),
            codec: None,
            bitrate: None,
            framerate: None,
            width: Some(1920),
            height: Some(1080),
            duration: None,
            file_size: 102400,
            mime_type: "image/webp".to_string(),
            extra: None,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: MediaMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(metadata.media_id, deserialized.media_id);
        assert_eq!(metadata.format, deserialized.format);
    }
}

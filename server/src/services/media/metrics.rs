use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Media event types for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaEventType {
    Upload,
    ProcessingStart,
    ProcessingComplete,
    ProcessingFailed,
    Access,
    Delete,
    MalwareScan,
    LifecycleAction,
}

/// Media event for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaEvent {
    pub event_type: MediaEventType,
    pub media_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub file_size: Option<u64>,
    pub duration_ms: Option<u64>,
    pub details: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl MediaEvent {
    pub fn upload(user_id: Uuid, media_id: Uuid, file_size: u64) -> Self {
        Self {
            event_type: MediaEventType::Upload,
            media_id: Some(media_id),
            user_id: Some(user_id),
            file_size: Some(file_size),
            duration_ms: None,
            details: None,
            timestamp: Utc::now(),
        }
    }

    pub fn processing_start(media_id: Uuid) -> Self {
        Self {
            event_type: MediaEventType::ProcessingStart,
            media_id: Some(media_id),
            user_id: None,
            file_size: None,
            duration_ms: None,
            details: None,
            timestamp: Utc::now(),
        }
    }

    pub fn processing_complete(media_id: Uuid, duration_ms: u64) -> Self {
        Self {
            event_type: MediaEventType::ProcessingComplete,
            media_id: Some(media_id),
            user_id: None,
            file_size: None,
            duration_ms: Some(duration_ms),
            details: None,
            timestamp: Utc::now(),
        }
    }

    pub fn processing_failed(media_id: Uuid, error: &str) -> Self {
        Self {
            event_type: MediaEventType::ProcessingFailed,
            media_id: Some(media_id),
            user_id: None,
            file_size: None,
            duration_ms: None,
            details: Some(error.to_string()),
            timestamp: Utc::now(),
        }
    }

    pub fn access(media_id: Uuid, user_id: Option<Uuid>) -> Self {
        Self {
            event_type: MediaEventType::Access,
            media_id: Some(media_id),
            user_id,
            file_size: None,
            duration_ms: None,
            details: None,
            timestamp: Utc::now(),
        }
    }
}


/// Real-time metrics for media system
#[derive(Debug, Default)]
pub struct MediaMetrics {
    // Upload metrics
    pub total_uploads: AtomicU64,
    pub total_upload_bytes: AtomicU64,
    pub failed_uploads: AtomicU64,
    
    // Processing metrics
    pub total_processed: AtomicU64,
    pub processing_errors: AtomicU64,
    pub total_processing_time_ms: AtomicU64,
    
    // Access metrics
    pub total_accesses: AtomicU64,
    pub total_bandwidth_bytes: AtomicU64,
    
    // Storage metrics
    pub current_storage_bytes: AtomicU64,
}

impl MediaMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_upload(&self, file_size: u64) {
        self.total_uploads.fetch_add(1, Ordering::Relaxed);
        self.total_upload_bytes.fetch_add(file_size, Ordering::Relaxed);
    }

    pub fn record_upload_failure(&self) {
        self.failed_uploads.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_processing(&self, duration_ms: u64) {
        self.total_processed.fetch_add(1, Ordering::Relaxed);
        self.total_processing_time_ms.fetch_add(duration_ms, Ordering::Relaxed);
    }

    pub fn record_processing_error(&self) {
        self.processing_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_access(&self, bytes: u64) {
        self.total_accesses.fetch_add(1, Ordering::Relaxed);
        self.total_bandwidth_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn get_snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_uploads: self.total_uploads.load(Ordering::Relaxed),
            total_upload_bytes: self.total_upload_bytes.load(Ordering::Relaxed),
            failed_uploads: self.failed_uploads.load(Ordering::Relaxed),
            total_processed: self.total_processed.load(Ordering::Relaxed),
            processing_errors: self.processing_errors.load(Ordering::Relaxed),
            avg_processing_time_ms: self.avg_processing_time(),
            total_accesses: self.total_accesses.load(Ordering::Relaxed),
            total_bandwidth_bytes: self.total_bandwidth_bytes.load(Ordering::Relaxed),
            current_storage_bytes: self.current_storage_bytes.load(Ordering::Relaxed),
            timestamp: Utc::now(),
        }
    }

    fn avg_processing_time(&self) -> u64 {
        let total = self.total_processed.load(Ordering::Relaxed);
        if total == 0 {
            return 0;
        }
        self.total_processing_time_ms.load(Ordering::Relaxed) / total
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub total_uploads: u64,
    pub total_upload_bytes: u64,
    pub failed_uploads: u64,
    pub total_processed: u64,
    pub processing_errors: u64,
    pub avg_processing_time_ms: u64,
    pub total_accesses: u64,
    pub total_bandwidth_bytes: u64,
    pub current_storage_bytes: u64,
    pub timestamp: DateTime<Utc>,
}

/// Media logger for audit and monitoring
pub struct MediaLogger {
    db_pool: Option<PgPool>,
    metrics: Arc<MediaMetrics>,
    events: Arc<RwLock<Vec<MediaEvent>>>,
    max_events: usize,
}

impl MediaLogger {
    pub fn new(db_pool: Option<PgPool>) -> Self {
        Self {
            db_pool,
            metrics: Arc::new(MediaMetrics::new()),
            events: Arc::new(RwLock::new(Vec::new())),
            max_events: 1000,
        }
    }

    /// Log an event
    pub async fn log_event(&self, event: MediaEvent) {
        // Log to tracing
        match &event.event_type {
            MediaEventType::Upload => {
                info!(
                    media_id = ?event.media_id,
                    user_id = ?event.user_id,
                    file_size = ?event.file_size,
                    "Media upload"
                );
                if let Some(size) = event.file_size {
                    self.metrics.record_upload(size);
                }
            }
            MediaEventType::ProcessingStart => {
                info!(media_id = ?event.media_id, "Processing started");
            }
            MediaEventType::ProcessingComplete => {
                info!(
                    media_id = ?event.media_id,
                    duration_ms = ?event.duration_ms,
                    "Processing completed"
                );
                if let Some(duration) = event.duration_ms {
                    self.metrics.record_processing(duration);
                }
            }
            MediaEventType::ProcessingFailed => {
                error!(
                    media_id = ?event.media_id,
                    error = ?event.details,
                    "Processing failed"
                );
                self.metrics.record_processing_error();
            }
            MediaEventType::Access => {
                debug!(media_id = ?event.media_id, "Media accessed");
                self.metrics.record_access(event.file_size.unwrap_or(0));
            }
            MediaEventType::Delete => {
                info!(media_id = ?event.media_id, user_id = ?event.user_id, "Media deleted");
            }
            MediaEventType::MalwareScan => {
                info!(media_id = ?event.media_id, result = ?event.details, "Malware scan");
            }
            MediaEventType::LifecycleAction => {
                info!(details = ?event.details, "Lifecycle action");
            }
        }

        // Store in memory buffer
        let mut events = self.events.write().await;
        if events.len() >= self.max_events {
            events.remove(0);
        }
        events.push(event.clone());

        // Optionally persist to database
        if let Some(ref pool) = self.db_pool {
            let _ = self.persist_event(pool, &event).await;
        }
    }

    /// Persist event to database
    async fn persist_event(&self, pool: &PgPool, event: &MediaEvent) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO media_audit_log (
                event_type, media_id, user_id, file_size, 
                duration_ms, details, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            format!("{:?}", event.event_type),
            event.media_id,
            event.user_id,
            event.file_size.map(|s| s as i64),
            event.duration_ms.map(|d| d as i64),
            event.details,
            event.timestamp.naive_utc()
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get metrics snapshot
    pub fn get_metrics(&self) -> MetricsSnapshot {
        self.metrics.get_snapshot()
    }

    /// Get recent events
    pub async fn get_recent_events(&self, limit: usize) -> Vec<MediaEvent> {
        let events = self.events.read().await;
        events.iter().rev().take(limit).cloned().collect()
    }

    /// Get metrics reference for direct updates
    pub fn metrics(&self) -> Arc<MediaMetrics> {
        self.metrics.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_event_creation() {
        let user_id = Uuid::new_v4();
        let media_id = Uuid::new_v4();
        
        let event = MediaEvent::upload(user_id, media_id, 1024);
        assert!(matches!(event.event_type, MediaEventType::Upload));
        assert_eq!(event.file_size, Some(1024));
    }

    #[test]
    fn test_metrics_recording() {
        let metrics = MediaMetrics::new();
        
        metrics.record_upload(1000);
        metrics.record_upload(2000);
        
        assert_eq!(metrics.total_uploads.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.total_upload_bytes.load(Ordering::Relaxed), 3000);
    }

    #[test]
    fn test_avg_processing_time() {
        let metrics = MediaMetrics::new();
        
        metrics.record_processing(100);
        metrics.record_processing(200);
        metrics.record_processing(300);
        
        let snapshot = metrics.get_snapshot();
        assert_eq!(snapshot.avg_processing_time_ms, 200);
    }

    #[tokio::test]
    async fn test_media_logger() {
        let logger = MediaLogger::new(None);
        
        let event = MediaEvent::upload(Uuid::new_v4(), Uuid::new_v4(), 1024);
        logger.log_event(event).await;
        
        let events = logger.get_recent_events(10).await;
        assert_eq!(events.len(), 1);
    }
}

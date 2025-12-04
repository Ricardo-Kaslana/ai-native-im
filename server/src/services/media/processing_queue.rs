use super::error::{MediaError, Result};
use super::image_processor::ImageProcessor;
use super::video_processor::VideoProcessor;
use super::models::{MediaType, MediaStatus};
use sqlx::PgPool;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Semaphore, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Maximum concurrent processing tasks per instance
const MAX_CONCURRENT_TASKS: usize = 10;

/// Maximum retry attempts for failed tasks
const MAX_RETRIES: u32 = 3;

/// Base delay for exponential backoff (in seconds)
const BASE_BACKOFF_SECS: u64 = 2;

/// Processing task
#[derive(Debug, Clone)]
pub struct ProcessingTask {
    pub media_id: Uuid,
    pub media_type: MediaType,
    pub input_path: String,
    pub retry_count: u32,
    pub queued_at: chrono::DateTime<chrono::Utc>,
}

impl ProcessingTask {
    /// Create a new processing task
    pub fn new(media_id: Uuid, media_type: MediaType, input_path: String) -> Self {
        Self {
            media_id,
            media_type,
            input_path,
            retry_count: 0,
            queued_at: chrono::Utc::now(),
        }
    }
}

/// Queue statistics
#[derive(Debug, Clone, Default)]
pub struct QueueStats {
    pub total_processed: u64,
    pub total_failed: u64,
    pub total_retried: u64,
    pub current_queue_size: usize,
    pub active_tasks: usize,
    pub available_slots: usize,
}

/// Processing queue for media files
pub struct ProcessingQueue {
    sender: mpsc::Sender<ProcessingTask>,
    semaphore: Arc<Semaphore>,
    stats: Arc<ProcessingStats>,
    queue_size: Arc<AtomicUsize>,
}

/// Internal statistics tracking
struct ProcessingStats {
    total_processed: AtomicU64,
    total_failed: AtomicU64,
    total_retried: AtomicU64,
}

impl ProcessingQueue {
    /// Create a new processing queue and start the worker
    pub fn new(
        db_pool: PgPool,
        image_processor: Arc<ImageProcessor>,
        video_processor: Arc<VideoProcessor>,
    ) -> Self {
        let (sender, receiver) = mpsc::channel::<ProcessingTask>(100);
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_TASKS));
        let stats = Arc::new(ProcessingStats {
            total_processed: AtomicU64::new(0),
            total_failed: AtomicU64::new(0),
            total_retried: AtomicU64::new(0),
        });
        let queue_size = Arc::new(AtomicUsize::new(0));

        // Start the worker
        let worker = ProcessingWorker {
            receiver,
            semaphore: semaphore.clone(),
            db_pool,
            image_processor,
            video_processor,
            stats: stats.clone(),
            queue_size: queue_size.clone(),
        };

        tokio::spawn(worker.run());

        Self { 
            sender, 
            semaphore,
            stats,
            queue_size,
        }
    }

    /// Queue a task for processing
    pub async fn queue_task(&self, task: ProcessingTask) -> Result<()> {
        // Check if queue is full
        if self.queue_size.load(Ordering::Relaxed) >= 100 {
            return Err(MediaError::InternalError(
                "Processing queue is full, please try again later".to_string()
            ));
        }

        self.queue_size.fetch_add(1, Ordering::Relaxed);
        
        self.sender
            .send(task)
            .await
            .map_err(|e| {
                self.queue_size.fetch_sub(1, Ordering::Relaxed);
                MediaError::InternalError(format!("Failed to queue task: {}", e))
            })?;

        Ok(())
    }

    /// Get the number of available processing slots
    pub fn available_slots(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Check if the queue is full
    pub fn is_full(&self) -> bool {
        self.semaphore.available_permits() == 0
    }

    /// Get current queue statistics
    pub fn get_stats(&self) -> QueueStats {
        QueueStats {
            total_processed: self.stats.total_processed.load(Ordering::Relaxed),
            total_failed: self.stats.total_failed.load(Ordering::Relaxed),
            total_retried: self.stats.total_retried.load(Ordering::Relaxed),
            current_queue_size: self.queue_size.load(Ordering::Relaxed),
            active_tasks: MAX_CONCURRENT_TASKS - self.semaphore.available_permits(),
            available_slots: self.semaphore.available_permits(),
        }
    }

    /// Get the current queue size
    pub fn queue_size(&self) -> usize {
        self.queue_size.load(Ordering::Relaxed)
    }
}

/// Worker that processes tasks from the queue
struct ProcessingWorker {
    receiver: mpsc::Receiver<ProcessingTask>,
    semaphore: Arc<Semaphore>,
    db_pool: PgPool,
    image_processor: Arc<ImageProcessor>,
    video_processor: Arc<VideoProcessor>,
    stats: Arc<ProcessingStats>,
    queue_size: Arc<AtomicUsize>,
}

impl ProcessingWorker {
    async fn run(mut self) {
        info!("Processing worker started with {} concurrent slots", MAX_CONCURRENT_TASKS);

        while let Some(task) = self.receiver.recv().await {
            // Decrement queue size as we're taking the task
            self.queue_size.fetch_sub(1, Ordering::Relaxed);

            let permit = match self.semaphore.clone().acquire_owned().await {
                Ok(permit) => permit,
                Err(e) => {
                    error!("Failed to acquire semaphore: {}", e);
                    continue;
                }
            };

            let db_pool = self.db_pool.clone();
            let image_processor = self.image_processor.clone();
            let video_processor = self.video_processor.clone();
            let semaphore = self.semaphore.clone();
            let stats = self.stats.clone();

            // Log queue wait time
            let wait_time = chrono::Utc::now()
                .signed_duration_since(task.queued_at)
                .num_milliseconds();
            debug!("Task {} waited {}ms in queue", task.media_id, wait_time);

            // Spawn task processing
            tokio::spawn(async move {
                let start_time = std::time::Instant::now();
                
                let result = process_task_with_retry(
                    task.clone(),
                    &db_pool,
                    &image_processor,
                    &video_processor,
                    &stats,
                )
                .await;

                let duration = start_time.elapsed();

                match result {
                    Ok(_) => {
                        stats.total_processed.fetch_add(1, Ordering::Relaxed);
                        info!(
                            "Task {} completed successfully in {:?}",
                            task.media_id, duration
                        );
                    }
                    Err(e) => {
                        stats.total_failed.fetch_add(1, Ordering::Relaxed);
                        error!(
                            "Task {} failed after {} retries in {:?}: {}",
                            task.media_id, task.retry_count, duration, e
                        );
                        // Update status to failed
                        let _ = update_media_status(&db_pool, task.media_id, MediaStatus::Failed).await;
                    }
                }

                // Release permit
                drop(permit);
                debug!(
                    "Released permit, available slots: {}",
                    semaphore.available_permits()
                );
            });
        }

        info!("Processing worker stopped");
    }
}

/// Process a task with retry logic
async fn process_task_with_retry(
    mut task: ProcessingTask,
    db_pool: &PgPool,
    image_processor: &ImageProcessor,
    video_processor: &VideoProcessor,
    stats: &ProcessingStats,
) -> Result<()> {
    loop {
        match process_task(&task, db_pool, image_processor, video_processor).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                task.retry_count += 1;
                stats.total_retried.fetch_add(1, Ordering::Relaxed);

                if task.retry_count >= MAX_RETRIES {
                    return Err(e);
                }

                // Exponential backoff: 2^retry_count seconds
                let delay = Duration::from_secs(BASE_BACKOFF_SECS.pow(task.retry_count));
                warn!(
                    "Task {} failed (attempt {}/{}), retrying in {:?}: {}",
                    task.media_id, task.retry_count, MAX_RETRIES, delay, e
                );

                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Process a single task
async fn process_task(
    task: &ProcessingTask,
    db_pool: &PgPool,
    image_processor: &ImageProcessor,
    video_processor: &VideoProcessor,
) -> Result<()> {
    info!("Processing task: {:?}", task);

    match task.media_type {
        MediaType::Image => {
            let result = image_processor.process_image(&task.input_path).await?;

            // Update database with processed info
            sqlx::query!(
                r#"
                UPDATE media_files
                SET 
                    processed_url = $1,
                    thumbnail_url = $2,
                    width = $3,
                    height = $4,
                    file_size = $5,
                    status = 'completed',
                    updated_at = NOW()
                WHERE id = $6
                "#,
                result.webp_path,
                result.thumbnail_path,
                result.width as i32,
                result.height as i32,
                result.file_size as i64,
                task.media_id
            )
            .execute(db_pool)
            .await?;
        }
        MediaType::Video => {
            let result = video_processor.process_video(&task.input_path).await?;

            // Update database with processed info
            sqlx::query!(
                r#"
                UPDATE media_files
                SET 
                    processed_url = $1,
                    thumbnail_url = $2,
                    width = $3,
                    height = $4,
                    duration = $5,
                    file_size = $6,
                    status = 'completed',
                    updated_at = NOW()
                WHERE id = $7
                "#,
                result.compressed_path,
                result.thumbnail_path,
                result.width as i32,
                result.height as i32,
                result.duration as f32,
                result.file_size as i64,
                task.media_id
            )
            .execute(db_pool)
            .await?;
        }
    }

    Ok(())
}

/// Update media status in database
async fn update_media_status(
    db_pool: &PgPool,
    media_id: Uuid,
    status: MediaStatus,
) -> Result<()> {
    let status_str = match status {
        MediaStatus::Processing => "processing",
        MediaStatus::Completed => "completed",
        MediaStatus::Failed => "failed",
    };

    sqlx::query!(
        r#"
        UPDATE media_files
        SET status = $1, updated_at = NOW()
        WHERE id = $2
        "#,
        status_str,
        media_id
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_concurrent_tasks() {
        assert_eq!(MAX_CONCURRENT_TASKS, 10);
    }

    #[test]
    fn test_max_retries() {
        assert_eq!(MAX_RETRIES, 3);
    }

    #[test]
    fn test_processing_task_creation() {
        let task = ProcessingTask::new(
            Uuid::new_v4(),
            MediaType::Image,
            "/tmp/test.jpg".to_string(),
        );

        assert_eq!(task.retry_count, 0);
        assert!(task.queued_at <= chrono::Utc::now());
    }

    #[test]
    fn test_queue_stats_default() {
        let stats = QueueStats::default();
        assert_eq!(stats.total_processed, 0);
        assert_eq!(stats.total_failed, 0);
        assert_eq!(stats.total_retried, 0);
        assert_eq!(stats.current_queue_size, 0);
        assert_eq!(stats.active_tasks, 0);
    }

    #[test]
    fn test_exponential_backoff() {
        // Test backoff calculation
        let base = BASE_BACKOFF_SECS;
        assert_eq!(base.pow(1), 2);  // First retry: 2 seconds
        assert_eq!(base.pow(2), 4);  // Second retry: 4 seconds
        assert_eq!(base.pow(3), 8);  // Third retry: 8 seconds
    }
}

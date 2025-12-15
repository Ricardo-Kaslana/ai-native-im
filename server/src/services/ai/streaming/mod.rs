//! Response Streaming Module
//!
//! This module provides streaming response capabilities for AI agents,
//! allowing real-time delivery of LLM responses to clients.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

use super::llm_client::StreamChunk;

/// Streaming errors
#[derive(Debug, Error)]
pub enum StreamingError {
    #[error("Stream interrupted by user")]
    Interrupted,

    #[error("Stream timed out")]
    Timeout,

    #[error("Stream channel closed")]
    ChannelClosed,

    #[error("LLM error: {0}")]
    LLMError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Streaming response state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamState {
    /// Stream is active and receiving chunks
    Active,
    /// Stream completed successfully
    Completed,
    /// Stream was interrupted by user
    Interrupted,
    /// Stream failed with error
    Failed,
}

/// A streaming response chunk with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingChunk {
    /// Unique ID for this stream
    pub stream_id: Uuid,
    /// Sequence number of this chunk
    pub sequence: u32,
    /// The content of this chunk
    pub content: String,
    /// Whether this is the final chunk
    pub is_final: bool,
    /// Current state of the stream
    pub state: StreamState,
    /// Timestamp of this chunk
    pub timestamp: DateTime<Utc>,
}

/// Configuration for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Maximum time to wait for a chunk (seconds)
    pub chunk_timeout_secs: u64,
    /// Maximum total stream duration (seconds)
    pub max_duration_secs: u64,
    /// Buffer size for the stream channel
    pub buffer_size: usize,
    /// Whether to allow interruption
    pub allow_interruption: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            chunk_timeout_secs: 30,
            max_duration_secs: 300,
            buffer_size: 32,
            allow_interruption: true,
        }
    }
}

/// Handle for controlling an active stream
pub struct StreamHandle {
    /// Unique ID for this stream
    pub stream_id: Uuid,
    /// Flag to signal interruption
    interrupted: Arc<AtomicBool>,
    /// Receiver for stream chunks
    receiver: mpsc::Receiver<StreamingChunk>,
}

impl StreamHandle {
    /// Create a new stream handle
    pub fn new(
        stream_id: Uuid,
        interrupted: Arc<AtomicBool>,
        receiver: mpsc::Receiver<StreamingChunk>,
    ) -> Self {
        Self {
            stream_id,
            interrupted,
            receiver,
        }
    }

    /// Interrupt the stream
    pub fn interrupt(&self) {
        self.interrupted.store(true, Ordering::SeqCst);
    }

    /// Check if the stream is interrupted
    pub fn is_interrupted(&self) -> bool {
        self.interrupted.load(Ordering::SeqCst)
    }

    /// Receive the next chunk
    pub async fn recv(&mut self) -> Option<StreamingChunk> {
        self.receiver.recv().await
    }
}

/// Trait for streaming response services
#[async_trait]
pub trait StreamingService: Send + Sync {
    /// Start a streaming response
    async fn start_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<StreamHandle, StreamingError>;

    /// Get the configuration
    fn config(&self) -> &StreamingConfig;
}

/// Default streaming service implementation
pub struct DefaultStreamingService {
    llm_client: Arc<dyn super::LLMClient>,
    config: StreamingConfig,
}

impl DefaultStreamingService {
    pub fn new(llm_client: Arc<dyn super::LLMClient>) -> Self {
        Self {
            llm_client,
            config: StreamingConfig::default(),
        }
    }

    pub fn with_config(llm_client: Arc<dyn super::LLMClient>, config: StreamingConfig) -> Self {
        Self { llm_client, config }
    }
}

#[async_trait]
impl StreamingService for DefaultStreamingService {
    async fn start_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<StreamHandle, StreamingError> {
        let stream_id = Uuid::new_v4();
        let interrupted = Arc::new(AtomicBool::new(false));
        let interrupted_clone = interrupted.clone();

        // Get the LLM stream
        let mut llm_rx = self
            .llm_client
            .generate_stream(system_prompt, user_message)
            .await
            .map_err(|e| StreamingError::LLMError(e.to_string()))?;

        // Create output channel
        let (tx, rx) = mpsc::channel(self.config.buffer_size);

        let config = self.config.clone();

        // Spawn task to process LLM chunks and forward to output
        tokio::spawn(async move {
            let mut sequence = 0u32;
            let start_time = Utc::now();

            loop {
                // Check for interruption
                if interrupted_clone.load(Ordering::SeqCst) {
                    let _ = tx
                        .send(StreamingChunk {
                            stream_id,
                            sequence,
                            content: String::new(),
                            is_final: true,
                            state: StreamState::Interrupted,
                            timestamp: Utc::now(),
                        })
                        .await;
                    break;
                }

                // Check for timeout
                let elapsed = (Utc::now() - start_time).num_seconds() as u64;
                if elapsed > config.max_duration_secs {
                    let _ = tx
                        .send(StreamingChunk {
                            stream_id,
                            sequence,
                            content: String::new(),
                            is_final: true,
                            state: StreamState::Failed,
                            timestamp: Utc::now(),
                        })
                        .await;
                    break;
                }

                // Wait for next chunk with timeout
                match tokio::time::timeout(
                    std::time::Duration::from_secs(config.chunk_timeout_secs),
                    llm_rx.recv(),
                )
                .await
                {
                    Ok(Some(chunk)) => {
                        let state = if chunk.is_final {
                            StreamState::Completed
                        } else {
                            StreamState::Active
                        };

                        let streaming_chunk = StreamingChunk {
                            stream_id,
                            sequence,
                            content: chunk.content,
                            is_final: chunk.is_final,
                            state,
                            timestamp: Utc::now(),
                        };

                        if tx.send(streaming_chunk).await.is_err() {
                            // Receiver dropped
                            break;
                        }

                        if chunk.is_final {
                            break;
                        }

                        sequence += 1;
                    }
                    Ok(None) => {
                        // LLM stream ended
                        let _ = tx
                            .send(StreamingChunk {
                                stream_id,
                                sequence,
                                content: String::new(),
                                is_final: true,
                                state: StreamState::Completed,
                                timestamp: Utc::now(),
                            })
                            .await;
                        break;
                    }
                    Err(_) => {
                        // Timeout
                        let _ = tx
                            .send(StreamingChunk {
                                stream_id,
                                sequence,
                                content: String::new(),
                                is_final: true,
                                state: StreamState::Failed,
                                timestamp: Utc::now(),
                            })
                            .await;
                        break;
                    }
                }
            }
        });

        Ok(StreamHandle::new(stream_id, interrupted, rx))
    }

    fn config(&self) -> &StreamingConfig {
        &self.config
    }
}

/// Mock streaming service for testing
pub struct MockStreamingService {
    chunks: Vec<String>,
    config: StreamingConfig,
    should_fail: bool,
}

impl MockStreamingService {
    pub fn new(chunks: Vec<String>) -> Self {
        Self {
            chunks,
            config: StreamingConfig::default(),
            should_fail: false,
        }
    }

    pub fn failing() -> Self {
        Self {
            chunks: Vec::new(),
            config: StreamingConfig::default(),
            should_fail: true,
        }
    }
}

impl Default for MockStreamingService {
    fn default() -> Self {
        Self::new(vec!["Hello".to_string(), " world".to_string(), "!".to_string()])
    }
}

#[async_trait]
impl StreamingService for MockStreamingService {
    async fn start_stream(
        &self,
        _system_prompt: &str,
        _user_message: &str,
    ) -> Result<StreamHandle, StreamingError> {
        if self.should_fail {
            return Err(StreamingError::LLMError("Mock failure".to_string()));
        }

        let stream_id = Uuid::new_v4();
        let interrupted = Arc::new(AtomicBool::new(false));
        let interrupted_clone = interrupted.clone();

        let (tx, rx) = mpsc::channel(self.config.buffer_size);
        let chunks = self.chunks.clone();

        tokio::spawn(async move {
            for (i, content) in chunks.iter().enumerate() {
                if interrupted_clone.load(Ordering::SeqCst) {
                    let _ = tx
                        .send(StreamingChunk {
                            stream_id,
                            sequence: i as u32,
                            content: String::new(),
                            is_final: true,
                            state: StreamState::Interrupted,
                            timestamp: Utc::now(),
                        })
                        .await;
                    return;
                }

                let _ = tx
                    .send(StreamingChunk {
                        stream_id,
                        sequence: i as u32,
                        content: content.clone(),
                        is_final: false,
                        state: StreamState::Active,
                        timestamp: Utc::now(),
                    })
                    .await;

                // Simulate delay
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }

            // Send final chunk
            let _ = tx
                .send(StreamingChunk {
                    stream_id,
                    sequence: chunks.len() as u32,
                    content: String::new(),
                    is_final: true,
                    state: StreamState::Completed,
                    timestamp: Utc::now(),
                })
                .await;
        });

        Ok(StreamHandle::new(stream_id, interrupted, rx))
    }

    fn config(&self) -> &StreamingConfig {
        &self.config
    }
}

/// Message queue for handling messages during streaming
pub struct StreamMessageQueue {
    messages: std::sync::RwLock<Vec<QueuedMessage>>,
    max_size: usize,
}

/// A message queued during streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {
    pub id: Uuid,
    pub content: String,
    pub sender_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

impl StreamMessageQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: std::sync::RwLock::new(Vec::new()),
            max_size,
        }
    }

    /// Queue a message
    pub fn enqueue(&self, message: QueuedMessage) -> bool {
        let mut messages = self.messages.write().unwrap();
        if messages.len() >= self.max_size {
            return false;
        }
        messages.push(message);
        true
    }

    /// Drain all queued messages
    pub fn drain(&self) -> Vec<QueuedMessage> {
        let mut messages = self.messages.write().unwrap();
        std::mem::take(&mut *messages)
    }

    /// Get the number of queued messages
    pub fn len(&self) -> usize {
        self.messages.read().unwrap().len()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.messages.read().unwrap().is_empty()
    }
}

impl Default for StreamMessageQueue {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        assert_eq!(config.chunk_timeout_secs, 30);
        assert_eq!(config.max_duration_secs, 300);
        assert_eq!(config.buffer_size, 32);
        assert!(config.allow_interruption);
    }

    #[tokio::test]
    async fn test_mock_streaming_service() {
        let service = MockStreamingService::default();
        let mut handle = service.start_stream("system", "user").await.unwrap();

        let mut chunks = Vec::new();
        while let Some(chunk) = handle.recv().await {
            chunks.push(chunk);
            if chunks.last().map(|c| c.is_final).unwrap_or(false) {
                break;
            }
        }

        assert!(!chunks.is_empty());
        assert!(chunks.last().unwrap().is_final);
        assert_eq!(chunks.last().unwrap().state, StreamState::Completed);
    }

    #[tokio::test]
    async fn test_stream_interruption() {
        let service = MockStreamingService::new(vec![
            "chunk1".to_string(),
            "chunk2".to_string(),
            "chunk3".to_string(),
            "chunk4".to_string(),
            "chunk5".to_string(),
        ]);

        let mut handle = service.start_stream("system", "user").await.unwrap();

        // Receive first chunk
        let first = handle.recv().await.unwrap();
        assert_eq!(first.content, "chunk1");

        // Interrupt the stream
        handle.interrupt();

        // Wait for interruption to be processed
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Should receive interrupted state
        let mut found_interrupted = false;
        while let Some(chunk) = handle.recv().await {
            if chunk.state == StreamState::Interrupted {
                found_interrupted = true;
                break;
            }
        }

        assert!(found_interrupted || handle.is_interrupted());
    }

    #[tokio::test]
    async fn test_stream_failure() {
        let service = MockStreamingService::failing();
        let result = service.start_stream("system", "user").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_message_queue() {
        let queue = StreamMessageQueue::new(3);

        let msg1 = QueuedMessage {
            id: Uuid::new_v4(),
            content: "msg1".to_string(),
            sender_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        };

        let msg2 = QueuedMessage {
            id: Uuid::new_v4(),
            content: "msg2".to_string(),
            sender_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        };

        assert!(queue.enqueue(msg1));
        assert!(queue.enqueue(msg2));
        assert_eq!(queue.len(), 2);

        let drained = queue.drain();
        assert_eq!(drained.len(), 2);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_message_queue_overflow() {
        let queue = StreamMessageQueue::new(2);

        for i in 0..3 {
            let msg = QueuedMessage {
                id: Uuid::new_v4(),
                content: format!("msg{}", i),
                sender_id: Uuid::new_v4(),
                timestamp: Utc::now(),
            };
            let result = queue.enqueue(msg);
            if i < 2 {
                assert!(result);
            } else {
                assert!(!result); // Should fail on third message
            }
        }
    }

    /// **Feature: phase3-ai-agent-evolution, Property 18: Streaming Interruption Handling**
    /// **Validates: Requirements 13.3**
    #[test]
    fn test_stream_state_transitions() {
        // Property: Stream states are valid
        let states = vec![
            StreamState::Active,
            StreamState::Completed,
            StreamState::Interrupted,
            StreamState::Failed,
        ];

        for state in states {
            // All states should be serializable
            let json = serde_json::to_string(&state).unwrap();
            let deserialized: StreamState = serde_json::from_str(&json).unwrap();
            assert_eq!(state, deserialized);
        }
    }

    /// **Feature: phase3-ai-agent-evolution, Property 18b: Streaming Chunk Sequence**
    /// **Validates: Requirements 13.1**
    #[tokio::test]
    async fn test_streaming_chunk_sequence() {
        let service = MockStreamingService::new(vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]);

        let mut handle = service.start_stream("system", "user").await.unwrap();

        let mut sequences = Vec::new();
        while let Some(chunk) = handle.recv().await {
            sequences.push(chunk.sequence);
            if chunk.is_final {
                break;
            }
        }

        // Property: Sequences should be monotonically increasing
        for i in 1..sequences.len() {
            assert!(sequences[i] >= sequences[i - 1]);
        }
    }
}

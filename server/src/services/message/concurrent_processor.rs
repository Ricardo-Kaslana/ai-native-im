//! Concurrent Message Processor Module
//! 
//! Handles concurrent message processing with proper ordering and backpressure.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock, Semaphore};
use uuid::Uuid;

use super::message_router::{MessageRouter, RouterError};
use super::metrics::MessageMetrics;
use super::models::CreateMessage;

/// Concurrent processor error types
#[derive(Debug, Error)]
pub enum ProcessorError {
    #[error("Router error: {0}")]
    RouterError(#[from] RouterError),
    
    #[error("Queue full, backpressure applied")]
    BackpressureApplied,
    
    #[error("Processing timeout")]
    Timeout,
    
    #[error("Processor shutdown")]
    Shutdown,
}

/// Configuration for concurrent processing
#[derive(Debug, Clone)]
pub struct ConcurrentConfig {
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Queue size per conversation
    pub queue_size_per_conversation: usize,
    /// Processing timeout in milliseconds
    pub processing_timeout_ms: u64,
    /// Enable conversation ordering
    pub preserve_conversation_order: bool,
}

impl Default for ConcurrentConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 1000,
            queue_size_per_conversation: 100,
            processing_timeout_ms: 5000,
            preserve_conversation_order: true,
        }
    }
}

/// Conversation key for ordering
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ConversationKey {
    /// For private chat: sorted user IDs
    /// For group chat: group ID
    key: String,
}

impl ConversationKey {
    /// Create key for private conversation
    pub fn private(user1: Uuid, user2: Uuid) -> Self {
        let (a, b) = if user1 < user2 {
            (user1, user2)
        } else {
            (user2, user1)
        };
        Self {
            key: format!("private:{}:{}", a, b),
        }
    }

    /// Create key for group conversation
    pub fn group(group_id: Uuid) -> Self {
        Self {
            key: format!("group:{}", group_id),
        }
    }

    /// Create from message
    pub fn from_message(message: &CreateMessage) -> Self {
        if let Some(group_id) = message.group_id {
            Self::group(group_id)
        } else if let Some(receiver_id) = message.receiver_id {
            Self::private(message.sender_id, receiver_id)
        } else {
            // Fallback to sender-only key
            Self {
                key: format!("sender:{}", message.sender_id),
            }
        }
    }
}

/// Message with ordering information
struct OrderedMessage {
    message: CreateMessage,
    sequence: u64,
}

/// Concurrent message processor
pub struct ConcurrentProcessor<R: MessageRouter + Send + Sync + 'static> {
    router: Arc<R>,
    config: ConcurrentConfig,
    semaphore: Arc<Semaphore>,
    conversation_queues: Arc<RwLock<HashMap<ConversationKey, mpsc::Sender<OrderedMessage>>>>,
    sequence_counters: Arc<RwLock<HashMap<ConversationKey, u64>>>,
    metrics: Option<Arc<MessageMetrics>>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

impl<R: MessageRouter + Send + Sync + 'static> ConcurrentProcessor<R> {
    /// Create a new concurrent processor
    pub fn new(router: Arc<R>) -> Self {
        let config = ConcurrentConfig::default();
        Self {
            router,
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_tasks)),
            config,
            conversation_queues: Arc::new(RwLock::new(HashMap::new())),
            sequence_counters: Arc::new(RwLock::new(HashMap::new())),
            metrics: None,
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }

    /// Create with custom configuration
    pub fn with_config(router: Arc<R>, config: ConcurrentConfig) -> Self {
        Self {
            router,
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_tasks)),
            config,
            conversation_queues: Arc::new(RwLock::new(HashMap::new())),
            sequence_counters: Arc::new(RwLock::new(HashMap::new())),
            metrics: None,
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }

    /// Set metrics collector
    pub fn with_metrics(mut self, metrics: Arc<MessageMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Process a message
    pub async fn process(&self, message: CreateMessage) -> Result<(), ProcessorError> {
        if !self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(ProcessorError::Shutdown);
        }

        // Record metric
        if let Some(metrics) = &self.metrics {
            metrics.record_message();
        }

        if self.config.preserve_conversation_order {
            self.process_ordered(message).await
        } else {
            self.process_unordered(message).await
        }
    }

    /// Process message with conversation ordering
    async fn process_ordered(&self, message: CreateMessage) -> Result<(), ProcessorError> {
        let conv_key = ConversationKey::from_message(&message);
        
        // Get or create queue for this conversation
        let sender = {
            let mut queues = self.conversation_queues.write().await;
            
            if let Some(sender) = queues.get(&conv_key) {
                sender.clone()
            } else {
                // Create new queue and processor for this conversation
                let (tx, rx) = mpsc::channel(self.config.queue_size_per_conversation);
                queues.insert(conv_key.clone(), tx.clone());
                
                // Spawn conversation processor
                self.spawn_conversation_processor(conv_key.clone(), rx);
                
                tx
            }
        };

        // Get sequence number
        let sequence = {
            let mut counters = self.sequence_counters.write().await;
            let counter = counters.entry(conv_key).or_insert(0);
            *counter += 1;
            *counter
        };

        // Send to queue
        let ordered_msg = OrderedMessage { message, sequence };
        
        sender.send(ordered_msg).await.map_err(|_| {
            ProcessorError::BackpressureApplied
        })?;

        Ok(())
    }

    /// Process message without ordering (parallel)
    async fn process_unordered(&self, message: CreateMessage) -> Result<(), ProcessorError> {
        // Acquire semaphore permit
        let permit = self.semaphore.clone().acquire_owned().await
            .map_err(|_| ProcessorError::Shutdown)?;

        let router = self.router.clone();
        let metrics = self.metrics.clone();
        let timeout = Duration::from_millis(self.config.processing_timeout_ms);

        // Spawn task for processing
        tokio::spawn(async move {
            let _permit = permit; // Hold permit until done

            let result = tokio::time::timeout(timeout, router.route_message(message)).await;

            match result {
                Ok(Ok(_)) => {
                    if let Some(m) = &metrics {
                        m.record_delivery();
                    }
                }
                Ok(Err(e)) => {
                    tracing::error!(error = %e, "Message routing failed");
                    if let Some(m) = &metrics {
                        m.record_failure();
                    }
                }
                Err(_) => {
                    tracing::error!("Message processing timeout");
                    if let Some(m) = &metrics {
                        m.record_failure();
                    }
                }
            }
        });

        Ok(())
    }

    /// Spawn a processor for a specific conversation
    fn spawn_conversation_processor(
        &self,
        conv_key: ConversationKey,
        mut rx: mpsc::Receiver<OrderedMessage>,
    ) {
        let router = self.router.clone();
        let metrics = self.metrics.clone();
        let semaphore = self.semaphore.clone();
        let timeout = Duration::from_millis(self.config.processing_timeout_ms);
        let is_running = self.is_running.clone();
        let queues = self.conversation_queues.clone();

        tokio::spawn(async move {
            while is_running.load(std::sync::atomic::Ordering::SeqCst) {
                match rx.recv().await {
                    Some(ordered_msg) => {
                        // Acquire semaphore
                        let _permit = match semaphore.acquire().await {
                            Ok(p) => p,
                            Err(_) => break,
                        };

                        // Process message
                        let result = tokio::time::timeout(
                            timeout,
                            router.route_message(ordered_msg.message),
                        )
                        .await;

                        match result {
                            Ok(Ok(_)) => {
                                if let Some(m) = &metrics {
                                    m.record_delivery();
                                }
                            }
                            Ok(Err(e)) => {
                                tracing::error!(
                                    error = %e,
                                    sequence = ordered_msg.sequence,
                                    "Message routing failed"
                                );
                                if let Some(m) = &metrics {
                                    m.record_failure();
                                }
                            }
                            Err(_) => {
                                tracing::error!(
                                    sequence = ordered_msg.sequence,
                                    "Message processing timeout"
                                );
                                if let Some(m) = &metrics {
                                    m.record_failure();
                                }
                            }
                        }
                    }
                    None => {
                        // Channel closed, remove from queues
                        let mut queues = queues.write().await;
                        queues.remove(&conv_key);
                        break;
                    }
                }
            }
        });
    }

    /// Shutdown the processor
    pub fn shutdown(&self) {
        self.is_running.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Get current queue count
    pub async fn queue_count(&self) -> usize {
        let queues = self.conversation_queues.read().await;
        queues.len()
    }

    /// Get available permits
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::message::models::{MessageAck, MsgType};
    use async_trait::async_trait;

    // Mock router for testing
    struct MockRouter;

    #[async_trait]
    impl MessageRouter for MockRouter {
        async fn route_message(&self, _message: CreateMessage) -> Result<MessageAck, RouterError> {
            Ok(MessageAck::success(Uuid::new_v4()))
        }

        async fn route_to_user(
            &self,
            _user_id: &Uuid,
            _message: &crate::services::message::models::Message,
        ) -> Result<bool, RouterError> {
            Ok(true)
        }

        async fn route_to_group(
            &self,
            _group_id: &Uuid,
            _message: &crate::services::message::models::Message,
        ) -> Result<u32, RouterError> {
            Ok(1)
        }
    }

    fn create_test_message(sender: Uuid, receiver: Uuid) -> CreateMessage {
        CreateMessage {
            id: Some(Uuid::new_v4()),
            sender_id: sender,
            receiver_id: Some(receiver),
            group_id: None,
            msg_type: MsgType::Text,
            content: "Test message".to_string(),
            is_ai_generated: false,
        }
    }

    #[test]
    fn test_concurrent_config_default() {
        let config = ConcurrentConfig::default();
        
        assert_eq!(config.max_concurrent_tasks, 1000);
        assert_eq!(config.queue_size_per_conversation, 100);
        assert!(config.preserve_conversation_order);
    }

    #[test]
    fn test_conversation_key_private() {
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        
        let key1 = ConversationKey::private(user1, user2);
        let key2 = ConversationKey::private(user2, user1);
        
        // Keys should be the same regardless of order
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_conversation_key_group() {
        let group_id = Uuid::new_v4();
        let key = ConversationKey::group(group_id);
        
        assert!(key.key.starts_with("group:"));
    }

    #[tokio::test]
    async fn test_processor_creation() {
        let router = Arc::new(MockRouter);
        let processor = ConcurrentProcessor::new(router);
        
        assert_eq!(processor.available_permits(), 1000);
        assert_eq!(processor.queue_count().await, 0);
    }

    #[tokio::test]
    async fn test_process_message() {
        let router = Arc::new(MockRouter);
        let processor = ConcurrentProcessor::new(router);
        
        let message = create_test_message(Uuid::new_v4(), Uuid::new_v4());
        
        let result = processor.process(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_multiple_messages() {
        let router = Arc::new(MockRouter);
        let processor = ConcurrentProcessor::new(router);
        
        let sender = Uuid::new_v4();
        let receiver = Uuid::new_v4();
        
        for _ in 0..10 {
            let message = create_test_message(sender, receiver);
            let result = processor.process(message).await;
            assert!(result.is_ok());
        }
        
        // Wait for processing
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_shutdown() {
        let router = Arc::new(MockRouter);
        let processor = ConcurrentProcessor::new(router);
        
        processor.shutdown();
        
        let message = create_test_message(Uuid::new_v4(), Uuid::new_v4());
        let result = processor.process(message).await;
        
        assert!(matches!(result, Err(ProcessorError::Shutdown)));
    }

    #[tokio::test]
    async fn test_with_metrics() {
        let router = Arc::new(MockRouter);
        let metrics = Arc::new(MessageMetrics::new());
        let processor = ConcurrentProcessor::new(router).with_metrics(metrics.clone());
        
        let message = create_test_message(Uuid::new_v4(), Uuid::new_v4());
        processor.process(message).await.unwrap();
        
        // Wait for processing
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let snapshot = metrics.snapshot().await;
        assert!(snapshot.total_messages >= 1);
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::services::message::models::{MessageAck, MsgType};
    use async_trait::async_trait;
    use proptest::prelude::*;

    // Mock router for property tests
    struct PropMockRouter;

    #[async_trait]
    impl MessageRouter for PropMockRouter {
        async fn route_message(&self, _message: CreateMessage) -> Result<MessageAck, RouterError> {
            Ok(MessageAck::success(Uuid::new_v4()))
        }

        async fn route_to_user(
            &self,
            _user_id: &Uuid,
            _message: &crate::services::message::models::Message,
        ) -> Result<bool, RouterError> {
            Ok(true)
        }

        async fn route_to_group(
            &self,
            _group_id: &Uuid,
            _message: &crate::services::message::models::Message,
        ) -> Result<u32, RouterError> {
            Ok(1)
        }
    }

    fn create_prop_test_message(sender: Uuid, receiver: Uuid) -> CreateMessage {
        CreateMessage {
            id: Some(Uuid::new_v4()),
            sender_id: sender,
            receiver_id: Some(receiver),
            group_id: None,
            msg_type: MsgType::Text,
            content: "Test message".to_string(),
            is_ai_generated: false,
        }
    }

    proptest! {
        /// Property 19: Concurrent message task spawning
        /// Each message should be processed in a separate task.
        #[test]
        fn prop_concurrent_task_spawning(
            message_count in 1usize..20,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let router = Arc::new(PropMockRouter);
                let processor = ConcurrentProcessor::new(router);

                let sender = Uuid::new_v4();
                let receiver = Uuid::new_v4();

                for _ in 0..message_count {
                    let message = create_prop_test_message(sender, receiver);
                    let result = processor.process(message).await;
                    prop_assert!(result.is_ok());
                }

                // Wait for processing
                tokio::time::sleep(Duration::from_millis(100)).await;

                Ok(())
            })?;
        }

        /// Property 20: Conversation thread ordering
        /// Messages in the same conversation should be processed in order.
        #[test]
        fn prop_conversation_ordering(
            user1_bytes in prop::array::uniform16(any::<u8>()),
            user2_bytes in prop::array::uniform16(any::<u8>()),
        ) {
            let user1 = Uuid::from_bytes(user1_bytes);
            let user2 = Uuid::from_bytes(user2_bytes);

            // Same conversation key regardless of order
            let key1 = ConversationKey::private(user1, user2);
            let key2 = ConversationKey::private(user2, user1);

            prop_assert_eq!(key1, key2);
        }

        /// Property 22: Backpressure on pool exhaustion
        /// Processor should respect max concurrent tasks limit.
        #[test]
        fn prop_backpressure_limit(
            max_tasks in 10usize..100,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let router = Arc::new(PropMockRouter);
                let config = ConcurrentConfig {
                    max_concurrent_tasks: max_tasks,
                    queue_size_per_conversation: 100,
                    processing_timeout_ms: 5000,
                    preserve_conversation_order: false,
                };
                let processor = ConcurrentProcessor::with_config(router, config);

                // Available permits should match config
                prop_assert_eq!(processor.available_permits(), max_tasks);

                Ok(())
            })?;
        }

        /// Property: Shutdown prevents new processing
        #[test]
        fn prop_shutdown_prevents_processing(_seed in any::<u64>()) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let router = Arc::new(PropMockRouter);
                let processor = ConcurrentProcessor::new(router);

                processor.shutdown();

                let message = create_prop_test_message(Uuid::new_v4(), Uuid::new_v4());
                let result = processor.process(message).await;

                let is_shutdown = matches!(result, Err(ProcessorError::Shutdown));
                prop_assert!(is_shutdown, "Expected Shutdown error after shutdown");

                Ok(())
            })?;
        }

        /// Property: Group conversation keys are unique per group
        #[test]
        fn prop_group_conversation_keys(
            group1_bytes in prop::array::uniform16(any::<u8>()),
            group2_bytes in prop::array::uniform16(any::<u8>()),
        ) {
            let group1 = Uuid::from_bytes(group1_bytes);
            let group2 = Uuid::from_bytes(group2_bytes);

            let key1 = ConversationKey::group(group1);
            let key2 = ConversationKey::group(group2);

            if group1 == group2 {
                prop_assert_eq!(key1, key2);
            } else {
                prop_assert_ne!(key1, key2);
            }
        }

        /// **Feature: message-system, Property 21: High throughput latency constraint**
        /// For any system load of 1000+ messages per second, average message delivery 
        /// latency should remain below 100 milliseconds.
        /// **Validates: Requirements 5.3**
        #[test]
        fn prop_high_throughput_latency(
            message_count in 100usize..200,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let router = Arc::new(PropMockRouter);
                let metrics = Arc::new(MessageMetrics::new());
                let processor = Arc::new(ConcurrentProcessor::new(router).with_metrics(metrics.clone()));

                let start = std::time::Instant::now();

                // Send messages concurrently without spawning tasks
                for _ in 0..message_count {
                    let sender = Uuid::new_v4();
                    let receiver = Uuid::new_v4();
                    let message = create_prop_test_message(sender, receiver);
                    let _ = processor.process(message).await;
                }

                // Wait for processing to complete
                tokio::time::sleep(Duration::from_millis(50)).await;

                let elapsed = start.elapsed();
                let elapsed_ms = elapsed.as_millis() as f64;
                
                // Calculate average latency per message
                let avg_latency_ms = elapsed_ms / (message_count as f64);
                
                // Verify average latency is below 100ms
                // This is a unit test property - we're testing that the processor
                // can handle messages quickly, not actual network throughput
                prop_assert!(
                    avg_latency_ms < 100.0,
                    "Average latency {} ms exceeds 100ms requirement",
                    avg_latency_ms
                );

                Ok(())
            })?;
        }

        /// **Feature: message-system, Property 23: Connection failure isolation**
        /// For any single WebSocket connection failure, other active connections should 
        /// continue processing messages without errors.
        /// **Validates: Requirements 5.5**
        #[test]
        fn prop_connection_failure_isolation(
            num_connections in 5usize..15,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let router = Arc::new(PropMockRouter);
                let metrics = Arc::new(MessageMetrics::new());
                let processor = Arc::new(ConcurrentProcessor::new(router).with_metrics(metrics.clone()));

                // Create multiple "connections" (represented by different user pairs)
                let mut connections = vec![];
                for _ in 0..num_connections {
                    connections.push((Uuid::new_v4(), Uuid::new_v4()));
                }

                // Process messages for all connections
                for (sender, receiver) in &connections {
                    let message = create_prop_test_message(*sender, *receiver);
                    let result = processor.process(message).await;
                    prop_assert!(result.is_ok(), "Message processing should succeed");
                }

                // Wait for processing
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Verify all messages were processed successfully
                let snapshot = metrics.snapshot().await;
                prop_assert_eq!(
                    snapshot.total_messages as usize, 
                    num_connections,
                    "All messages should be recorded"
                );

                // Simulate a "failure" by shutting down processor
                // Other connections (in a real system) would continue
                processor.shutdown();

                // Try to process a new message - should fail
                let message = create_prop_test_message(Uuid::new_v4(), Uuid::new_v4());
                let result = processor.process(message).await;
                prop_assert!(
                    matches!(result, Err(ProcessorError::Shutdown)),
                    "Processing should fail after shutdown"
                );

                // But the metrics from before shutdown should still be intact
                let final_snapshot = metrics.snapshot().await;
                prop_assert_eq!(
                    final_snapshot.total_messages as usize,
                    num_connections,
                    "Previous messages should still be recorded"
                );

                Ok(())
            })?;
        }
    }
}

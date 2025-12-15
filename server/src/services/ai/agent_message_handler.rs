//! AI Agent Message Handler
//!
//! Handles AI Agent responses to group messages.

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::{AIAgentError, AIAgentService};
use super::cot::CoTConfig;
use super::intent::{IntentClassifier, IntentResult, ResponsePriority};

/// Message for AI Agent processing
#[derive(Debug, Clone)]
pub struct AgentMessage {
    pub message_id: Uuid,
    pub group_id: Uuid,
    pub sender_id: Uuid,
    pub sender_name: String,
    pub content: String,
    pub timestamp: i64,
}

/// AI Agent response
#[derive(Debug, Clone)]
pub struct AgentResponse {
    pub agent_id: Uuid,
    pub agent_name: String,
    pub group_id: Uuid,
    pub content: String,
    pub reply_to_message_id: Uuid,
    pub typing_delay_ms: u64,
    /// Detected user intent (if classified)
    pub intent: Option<IntentResult>,
}

/// Callback for sending agent responses
pub type ResponseCallback = Arc<dyn Fn(AgentResponse) + Send + Sync>;

/// AI Agent Message Handler
///
/// Processes incoming group messages and generates AI responses when appropriate.
pub struct AgentMessageHandler {
    agent_service: AIAgentService,
    response_tx: Option<mpsc::UnboundedSender<AgentResponse>>,
    /// Chain of Thought configuration
    cot_config: Option<CoTConfig>,
    /// Intent classifier
    intent_classifier: IntentClassifier,
}

impl AgentMessageHandler {
    /// Create a new agent message handler
    pub fn new(agent_service: AIAgentService) -> Self {
        Self {
            agent_service,
            response_tx: None,
            cot_config: Some(CoTConfig::default()), // CoT enabled by default
            intent_classifier: IntentClassifier::new(),
        }
    }

    /// Set the response channel
    pub fn with_response_channel(mut self, tx: mpsc::UnboundedSender<AgentResponse>) -> Self {
        self.response_tx = Some(tx);
        self
    }

    /// Set CoT configuration
    pub fn with_cot_config(mut self, config: CoTConfig) -> Self {
        self.cot_config = Some(config);
        self
    }

    /// Disable CoT
    pub fn without_cot(mut self) -> Self {
        self.cot_config = None;
        self
    }

    /// Process an incoming group message
    ///
    /// Returns the trigger decision and optionally generates a response.
    pub async fn process_message(
        &self,
        message: AgentMessage,
        recent_messages: &[String],
    ) -> Result<Option<AgentResponse>, AIAgentError> {
        info!(
            "🔍 AgentMessageHandler.process_message called for message_id: {}",
            message.message_id
        );
        
        // Get agent for this group
        let agent = match self.agent_service.get_agent_for_group(message.group_id).await? {
            Some(agent) if agent.is_active => agent,
            _ => {
                debug!("No active agent for group {}", message.group_id);
                return Ok(None);
            }
        };

        // Get last message time for silence detection
        let last_message_time = if message.timestamp > 0 {
            Some(chrono::DateTime::from_timestamp_millis(message.timestamp - 1).unwrap_or_default())
        } else {
            None
        };

        // Process message and evaluate trigger
        let (decision, historical_context) = self
            .agent_service
            .process_message(
                &agent,
                message.message_id,
                message.sender_id,
                &message.content,
                last_message_time,
            )
            .await?;

        // Log trigger evaluation
        if let Err(e) = self
            .agent_service
            .log_trigger(agent.id, message.group_id, message.message_id, &decision)
            .await
        {
            warn!("Failed to log trigger: {}", e);
        }

        // If should not respond, return early
        if !decision.should_respond {
            debug!(
                "Agent {} decided not to respond (trigger: {:?}, prob: {:.2})",
                agent.name, decision.trigger_type, decision.probability
            );
            return Ok(None);
        }

        info!(
            "Agent {} will respond to message (trigger: {:?}, prob: {:.2})",
            agent.name, decision.trigger_type, decision.probability
        );

        // Classify user intent
        let intent_result = self.intent_classifier.classify(&message.content);
        debug!(
            "Intent classified: {:?} (confidence: {:.0}%)",
            intent_result.primary_intent,
            intent_result.confidence * 100.0
        );

        // Adjust response priority based on intent
        let should_prioritize = matches!(
            intent_result.strategy.priority,
            ResponsePriority::High
        );
        if should_prioritize {
            debug!("High priority intent detected, processing urgently");
        }

        // Generate response with CoT
        let response_content = self
            .agent_service
            .generate_response_with_cot(
                &agent, 
                &message.content, 
                recent_messages, 
                &historical_context,
                self.cot_config.as_ref(),
            )
            .await?;

        // Calculate typing delay
        let typing_delay = self.agent_service.calculate_typing_delay(&response_content);

        let response = AgentResponse {
            agent_id: agent.id,
            agent_name: agent.name.clone(),
            group_id: message.group_id,
            content: response_content,
            reply_to_message_id: message.message_id,
            typing_delay_ms: typing_delay.as_millis() as u64,
            intent: Some(intent_result),
        };

        // Send response through channel if available
        if let Some(tx) = &self.response_tx {
            info!(
                "📤 Sending response to channel for message_id: {}, reply_to: {}",
                message.message_id, response.reply_to_message_id
            );
            if let Err(e) = tx.send(response.clone()) {
                error!("Failed to send agent response: {}", e);
            }
        }

        // Update statistics
        let response_time_ms = typing_delay.as_millis() as i32;
        if let Err(e) = self
            .agent_service
            .update_message_stats(agent.id, response_time_ms)
            .await
        {
            warn!("Failed to update message stats: {}", e);
        }

        Ok(Some(response))
    }

    /// Get the agent service reference
    pub fn agent_service(&self) -> &AIAgentService {
        &self.agent_service
    }
}

/// Background worker for processing agent responses
pub struct AgentResponseWorker {
    response_rx: mpsc::UnboundedReceiver<AgentResponse>,
    message_sender: Arc<dyn AgentMessageSender>,
    /// Track recently processed message IDs to prevent duplicates
    processed_messages: std::collections::HashSet<Uuid>,
}

/// Trait for sending agent messages back to the chat
#[async_trait::async_trait]
pub trait AgentMessageSender: Send + Sync {
    async fn send_message(
        &self,
        group_id: Uuid,
        agent_id: Uuid,
        agent_name: &str,
        content: &str,
        reply_to: Option<Uuid>,
    ) -> Result<Uuid, AIAgentError>;
}

impl AgentResponseWorker {
    /// Create a new response worker
    pub fn new(
        response_rx: mpsc::UnboundedReceiver<AgentResponse>,
        message_sender: Arc<dyn AgentMessageSender>,
    ) -> Self {
        Self {
            response_rx,
            message_sender,
            processed_messages: std::collections::HashSet::new(),
        }
    }

    /// Run the worker (processes responses with typing delay)
    pub async fn run(mut self) {
        info!("Agent response worker started");

        while let Some(response) = self.response_rx.recv().await {
            info!(
                "📥 AgentResponseWorker received response for reply_to: {}",
                response.reply_to_message_id
            );
            
            // Check for duplicate response (same reply_to_message_id means same trigger)
            if self.processed_messages.contains(&response.reply_to_message_id) {
                info!(
                    "⏭️ Skipping duplicate response for message {}",
                    response.reply_to_message_id
                );
                continue;
            }
            
            // Mark as processed
            self.processed_messages.insert(response.reply_to_message_id);
            info!(
                "✅ Processing response for reply_to: {}, total processed: {}",
                response.reply_to_message_id, self.processed_messages.len()
            );
            
            // Clean up old entries if too many (keep last 1000)
            if self.processed_messages.len() > 1000 {
                self.processed_messages.clear();
            }

            // Simulate typing delay
            if response.typing_delay_ms > 0 {
                debug!(
                    "Agent {} typing for {}ms",
                    response.agent_name, response.typing_delay_ms
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(response.typing_delay_ms))
                    .await;
            }

            // Send the message
            match self
                .message_sender
                .send_message(
                    response.group_id,
                    response.agent_id,
                    &response.agent_name,
                    &response.content,
                    Some(response.reply_to_message_id),
                )
                .await
            {
                Ok(message_id) => {
                    info!(
                        "Agent {} sent message {} to group {} (reply_to: {})",
                        response.agent_name, message_id, response.group_id, response.reply_to_message_id
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to send agent message to group {}: {}",
                        response.group_id, e
                    );
                }
            }
        }

        info!("Agent response worker stopped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{MockEmbeddingService, MockLLMClient, MockVectorStore};
    use std::sync::atomic::{AtomicU32, Ordering};

    struct MockMessageSender {
        send_count: AtomicU32,
    }

    impl MockMessageSender {
        fn new() -> Self {
            Self {
                send_count: AtomicU32::new(0),
            }
        }

        fn get_send_count(&self) -> u32 {
            self.send_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl AgentMessageSender for MockMessageSender {
        async fn send_message(
            &self,
            _group_id: Uuid,
            _agent_id: Uuid,
            _agent_name: &str,
            _content: &str,
            _reply_to: Option<Uuid>,
        ) -> Result<Uuid, AIAgentError> {
            self.send_count.fetch_add(1, Ordering::SeqCst);
            Ok(Uuid::new_v4())
        }
    }

    #[tokio::test]
    async fn test_response_worker() {
        let (tx, rx) = mpsc::unbounded_channel();
        let sender = Arc::new(MockMessageSender::new());

        let worker = AgentResponseWorker::new(rx, sender.clone());

        // Spawn worker
        let worker_handle = tokio::spawn(worker.run());

        // Send a response
        tx.send(AgentResponse {
            agent_id: Uuid::new_v4(),
            agent_name: "TestBot".to_string(),
            group_id: Uuid::new_v4(),
            content: "Hello!".to_string(),
            reply_to_message_id: Uuid::new_v4(),
            typing_delay_ms: 10, // Short delay for test
            intent: None,
        })
        .unwrap();

        // Wait a bit for processing
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Drop sender to stop worker
        drop(tx);

        // Wait for worker to finish
        let _ = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            worker_handle,
        )
        .await;

        assert_eq!(sender.get_send_count(), 1);
    }
}

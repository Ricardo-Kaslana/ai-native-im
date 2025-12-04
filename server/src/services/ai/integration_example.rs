//! AI Agent Integration Example
//!
//! This module demonstrates how to integrate AI Agent with the message routing system.
//! Supports multiple LLM providers: OpenAI, Anthropic Claude, Google Gemini, SiliconFlow, and custom/local models.
//!
//! # Quick Start with LLMClientFactory
//!
//! The easiest way to create an LLM client is using the factory:
//!
//! ```rust,ignore
//! use ai_im_server::services::LLMClientFactory;
//!
//! // Automatically detect and create client from environment variables
//! let client = LLMClientFactory::from_env();
//!
//! // Or create specific providers
//! let openai = LLMClientFactory::openai("sk-...".to_string(), Some("gpt-4".to_string()));
//! let anthropic = LLMClientFactory::anthropic("sk-ant-...".to_string(), Some("claude-3-opus".to_string()));
//! let google = LLMClientFactory::google("...".to_string(), Some("gemini-pro".to_string()));
//! let siliconflow = LLMClientFactory::siliconflow("sk-...".to_string(), Some("Qwen/Qwen2.5-7B-Instruct".to_string()));
//! ```
//!
//! # Full Example with Multiple Providers
//!
//! ```rust,ignore
//! use ai_im_server::services::{
//!     AIAgentService, AgentMessageHandler, AgentResponseWorker,
//!     LLMClientFactory, OpenAIClient, AnthropicClient, GoogleGeminiClient,
//!     OpenAIEmbedding, QdrantVectorStore, ChatMessage,
//! };
//! use std::sync::Arc;
//! use tokio::sync::mpsc;
//!
//! async fn setup_ai_agent_system(pool: sqlx::PgPool) {
//!     // 1. Create LLM client (auto-detect from environment)
//!     let llm_client = LLMClientFactory::from_env()
//!         .expect("No LLM provider configured");
//!
//!     // Or manually create specific provider:
//!     // OpenAI
//!     let openai_client = LLMClientFactory::openai(
//!         std::env::var("OPENAI_API_KEY").unwrap(),
//!         Some("gpt-4".to_string()),
//!     );
//!
//!     // Anthropic Claude
//!     let anthropic_client = LLMClientFactory::anthropic(
//!         std::env::var("ANTHROPIC_API_KEY").unwrap(),
//!         Some("claude-3-opus-20240229".to_string()),
//!     );
//!
//!     // Google Gemini
//!     let google_client = LLMClientFactory::google(
//!         std::env::var("GOOGLE_API_KEY").unwrap(),
//!         Some("gemini-pro".to_string()),
//!     );
//!
//!     // SiliconFlow (硅基流动) - Chinese AI Platform
//!     let siliconflow_client = LLMClientFactory::siliconflow(
//!         std::env::var("SILICONFLOW_API_KEY").unwrap(),
//!         Some("Qwen/Qwen2.5-7B-Instruct".to_string()), // or deepseek-ai/DeepSeek-V2.5
//!     );
//!
//!     // Custom/Local (Ollama, vLLM, etc.)
//!     let local_client = LLMClientFactory::openai_compatible(
//!         "".to_string(), // No API key needed for local
//!         "http://localhost:11434/v1".to_string(),
//!         "llama2".to_string(),
//!     );
//!
//!     // 2. Create embedding service (OpenAI or SiliconFlow)
//!     // OpenAI embedding
//!     let embedding_service = Arc::new(OpenAIEmbedding::new(
//!         std::env::var("OPENAI_API_KEY").unwrap(),
//!         None,
//!         None,
//!     ));
//!     // Or SiliconFlow embedding (better for Chinese text)
//!     // let embedding_service = Arc::new(SiliconFlowEmbedding::new(
//!     //     std::env::var("SILICONFLOW_API_KEY").unwrap(),
//!     //     Some("BAAI/bge-large-zh-v1.5".to_string()), // Chinese optimized
//!     // ));
//!
//!     // 3. Create vector store
//!     let vector_store = Arc::new(
//!         QdrantVectorStore::new("http://localhost:6333")
//!             .await
//!             .unwrap()
//!     );
//!
//!     // 4. Create AI Agent service with all components
//!     let agent_service = AIAgentService::new(pool)
//!         .with_llm_client(llm_client)
//!         .with_embedding_service(embedding_service)
//!         .with_vector_store(vector_store);
//!
//!     // 5. Create response channel
//!     let (response_tx, response_rx) = mpsc::unbounded_channel();
//!
//!     // 6. Create message handler
//!     let handler = AgentMessageHandler::new(agent_service)
//!         .with_response_channel(response_tx);
//!
//!     // 7. Create and spawn response worker
//!     let message_sender = Arc::new(YourMessageSender::new(/* ... */));
//!     let worker = AgentResponseWorker::new(response_rx, message_sender);
//!     tokio::spawn(worker.run());
//! }
//! ```
//!
//! # Using Conversation History
//!
//! ```rust,ignore
//! use ai_im_server::services::{LLMClient, ChatMessage, LLMClientFactory};
//!
//! async fn chat_with_history(client: &dyn LLMClient) {
//!     let messages = vec![
//!         ChatMessage::system("You are a helpful assistant."),
//!         ChatMessage::user("What's the weather like?"),
//!         ChatMessage::assistant("I don't have access to real-time weather data."),
//!         ChatMessage::user("Can you tell me a joke instead?"),
//!     ];
//!
//!     let response = client.generate_with_history(&messages).await.unwrap();
//!     println!("Response: {}", response);
//! }
//! ```
//!
//! # Streaming Responses
//!
//! ```rust,ignore
//! use ai_im_server::services::{LLMClient, LLMClientFactory};
//!
//! async fn stream_response(client: &dyn LLMClient) {
//!     let mut rx = client.generate_stream(
//!         "You are a helpful assistant.",
//!         "Tell me a story."
//!     ).await.unwrap();
//!
//!     while let Some(chunk) = rx.recv().await {
//!         if chunk.is_final {
//!             println!("\n[Done]");
//!             break;
//!         }
//!         print!("{}", chunk.content);
//!     }
//! }
//! ```
//!
//! # Integration Points
//!
//! 1. **Message Router**: Call `handler.process_message()` when routing group messages
//! 2. **WebSocket Handler**: Send typing indicators before agent responds
//! 3. **Message Persister**: Store AI-generated messages with `is_ai_generated = true`
//!
//! # Configuration
//!
//! Set ONE of the following provider configurations:
//!
//! ## OpenAI (recommended)
//! - `OPENAI_API_KEY`: OpenAI API key
//! - `OPENAI_MODEL`: Model name (default: gpt-3.5-turbo)
//! - `OPENAI_API_BASE`: Custom endpoint (optional, for Azure OpenAI)
//!
//! ## Anthropic Claude
//! - `ANTHROPIC_API_KEY`: Anthropic API key
//! - `ANTHROPIC_MODEL`: Model name (default: claude-3-haiku-20240307)
//!
//! ## Google Gemini
//! - `GOOGLE_API_KEY`: Google API key
//! - `GOOGLE_MODEL`: Model name (default: gemini-pro)
//!
//! ## SiliconFlow 硅基流动 (Chinese AI Platform)
//! - `SILICONFLOW_API_KEY`: SiliconFlow API key (get at https://cloud.siliconflow.cn/)
//! - `SILICONFLOW_MODEL`: Model name (default: Qwen/Qwen2.5-7B-Instruct)
//!   - Popular models: Qwen/Qwen2.5-7B-Instruct, deepseek-ai/DeepSeek-V2.5, THUDM/glm-4-9b-chat
//!
//! ## Custom/Local (OpenAI-compatible)
//! - `LLM_API_BASE`: API endpoint (e.g., http://localhost:11434/v1 for Ollama)
//! - `LLM_MODEL`: Model name
//! - `LLM_API_KEY`: API key (optional)
//!
//! ## Additional
//! - `QDRANT_URL`: Qdrant vector database URL (default: http://localhost:6333)
//! - `OPENAI_EMBEDDING_MODEL`: OpenAI embedding model (default: text-embedding-3-small)
//! - `SILICONFLOW_EMBEDDING_MODEL`: SiliconFlow embedding model (default: BAAI/bge-large-zh-v1.5)
//! - `SILICONFLOW_VISION_MODEL`: SiliconFlow vision model (default: Qwen/Qwen2-VL-7B-Instruct)
//! - `ENABLE_VISION`: Enable vision capabilities (requires OpenAI or SiliconFlow)

use super::{AgentMessage, AgentMessageSender, AIAgentError};
use crate::services::message::models::Message;
use uuid::Uuid;

/// Helper to convert a Message to AgentMessage
pub fn message_to_agent_message(
    message: &Message,
    sender_name: String,
) -> Option<AgentMessage> {
    let group_id = message.group_id?;
    
    Some(AgentMessage {
        message_id: message.id,
        group_id,
        sender_id: message.sender_id,
        sender_name,
        content: message.content.clone(),
        timestamp: message.timestamp,
    })
}

/// Simple message sender that uses a callback
pub struct CallbackMessageSender<F>
where
    F: Fn(Uuid, Uuid, String, String, Option<Uuid>) -> Result<Uuid, AIAgentError> + Send + Sync,
{
    callback: F,
}

impl<F> CallbackMessageSender<F>
where
    F: Fn(Uuid, Uuid, String, String, Option<Uuid>) -> Result<Uuid, AIAgentError> + Send + Sync,
{
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

#[async_trait::async_trait]
impl<F> AgentMessageSender for CallbackMessageSender<F>
where
    F: Fn(Uuid, Uuid, String, String, Option<Uuid>) -> Result<Uuid, AIAgentError> + Send + Sync,
{
    async fn send_message(
        &self,
        group_id: Uuid,
        agent_id: Uuid,
        agent_name: &str,
        content: &str,
        reply_to: Option<Uuid>,
    ) -> Result<Uuid, AIAgentError> {
        (self.callback)(
            group_id,
            agent_id,
            agent_name.to_string(),
            content.to_string(),
            reply_to,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::message::models::MsgType;

    #[test]
    fn test_message_to_agent_message() {
        let message = Message {
            id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            receiver_id: None,
            group_id: Some(Uuid::new_v4()),
            msg_type: MsgType::Text as i16,
            content: "Hello!".to_string(),
            timestamp: 1234567890,
            is_ai_generated: false,
            delivered: false,
            created_at: chrono::Utc::now(),
        };

        let agent_msg = message_to_agent_message(&message, "TestUser".to_string());
        assert!(agent_msg.is_some());

        let agent_msg = agent_msg.unwrap();
        assert_eq!(agent_msg.message_id, message.id);
        assert_eq!(agent_msg.group_id, message.group_id.unwrap());
        assert_eq!(agent_msg.sender_id, message.sender_id);
        assert_eq!(agent_msg.content, "Hello!");
    }

    #[test]
    fn test_message_to_agent_message_no_group() {
        let message = Message {
            id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            receiver_id: Some(Uuid::new_v4()),
            group_id: None, // Private message
            msg_type: MsgType::Text as i16,
            content: "Hello!".to_string(),
            timestamp: 1234567890,
            is_ai_generated: false,
            delivered: false,
            created_at: chrono::Utc::now(),
        };

        let agent_msg = message_to_agent_message(&message, "TestUser".to_string());
        assert!(agent_msg.is_none()); // Should be None for private messages
    }
}

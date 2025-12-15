//! Summarization Service Module
//!
//! Provides message summarization capabilities for chat groups.
//!
//! Features:
//! - Summarize recent messages in a group
//! - Configurable message count and time range
//! - Caching with expiration
//! - Support for different summary styles

mod service;

pub use service::*;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AIAgentError;

/// Summary request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryRequest {
    /// Group ID to summarize
    pub group_id: Uuid,
    /// Maximum number of messages to include (default: 100)
    #[serde(default = "default_message_count")]
    pub message_count: usize,
    /// Only include messages after this time (optional)
    pub since: Option<DateTime<Utc>>,
    /// Summary style
    #[serde(default)]
    pub style: SummaryStyle,
    /// Whether to use cache
    #[serde(default = "default_use_cache")]
    pub use_cache: bool,
}

fn default_message_count() -> usize {
    100
}

fn default_use_cache() -> bool {
    true
}

impl Default for SummaryRequest {
    fn default() -> Self {
        Self {
            group_id: Uuid::nil(),
            message_count: default_message_count(),
            since: None,
            style: SummaryStyle::default(),
            use_cache: default_use_cache(),
        }
    }
}

/// Summary style options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryStyle {
    /// Brief bullet points
    #[default]
    Brief,
    /// Detailed narrative
    Detailed,
    /// Key topics only
    Topics,
    /// Action items and decisions
    ActionItems,
}

impl SummaryStyle {
    /// Get the prompt instruction for this style
    pub fn prompt_instruction(&self) -> &'static str {
        match self {
            SummaryStyle::Brief => {
                "请用简洁的要点形式总结以下对话内容，突出主要话题和结论。"
            }
            SummaryStyle::Detailed => {
                "请详细总结以下对话内容，包括讨论的背景、主要观点、不同意见和最终结论。"
            }
            SummaryStyle::Topics => {
                "请列出以下对话中讨论的主要话题，每个话题用一句话描述。"
            }
            SummaryStyle::ActionItems => {
                "请从以下对话中提取所有待办事项、决定和行动计划。"
            }
        }
    }
}

/// Generated summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    /// Unique summary ID
    pub id: Uuid,
    /// Group ID
    pub group_id: Uuid,
    /// Summary content
    pub content: String,
    /// Number of messages summarized
    pub message_count: usize,
    /// Time range start
    pub time_range_start: Option<DateTime<Utc>>,
    /// Time range end
    pub time_range_end: Option<DateTime<Utc>>,
    /// Summary style used
    pub style: SummaryStyle,
    /// When the summary was generated
    pub generated_at: DateTime<Utc>,
    /// Cache expiration time
    pub expires_at: DateTime<Utc>,
}

/// Summarization service trait
#[async_trait]
pub trait SummarizeService: Send + Sync {
    /// Generate a summary for the given request
    async fn summarize(&self, request: SummaryRequest) -> Result<Summary, AIAgentError>;

    /// Get cached summary if available and not expired
    async fn get_cached(&self, group_id: Uuid, style: SummaryStyle) -> Option<Summary>;

    /// Invalidate cached summary for a group
    async fn invalidate_cache(&self, group_id: Uuid) -> Result<(), AIAgentError>;
}

/// Message data for summarization
#[derive(Debug, Clone)]
pub struct MessageForSummary {
    pub id: Uuid,
    pub sender_name: String,
    pub content: String,
    pub sent_at: DateTime<Utc>,
}

impl MessageForSummary {
    /// Format message for LLM context
    pub fn format(&self) -> String {
        format!(
            "[{}] {}: {}",
            self.sent_at.format("%H:%M"),
            self.sender_name,
            self.content
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_style_prompt() {
        assert!(!SummaryStyle::Brief.prompt_instruction().is_empty());
        assert!(!SummaryStyle::Detailed.prompt_instruction().is_empty());
        assert!(!SummaryStyle::Topics.prompt_instruction().is_empty());
        assert!(!SummaryStyle::ActionItems.prompt_instruction().is_empty());
    }

    #[test]
    fn test_message_format() {
        let msg = MessageForSummary {
            id: Uuid::new_v4(),
            sender_name: "Alice".to_string(),
            content: "Hello, world!".to_string(),
            sent_at: Utc::now(),
        };
        let formatted = msg.format();
        assert!(formatted.contains("Alice"));
        assert!(formatted.contains("Hello, world!"));
    }

    #[test]
    fn test_summary_request_default() {
        let request = SummaryRequest::default();
        assert_eq!(request.message_count, 100);
        assert!(request.use_cache);
        assert_eq!(request.style, SummaryStyle::Brief);
    }
}

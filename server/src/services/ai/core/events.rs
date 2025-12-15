//! Agent Events and Actions
//!
//! This module defines the core event and action types for the AI Agent system.
//! These types form the interface between the three layers of the cognitive architecture:
//! - AgentEvent: Input from Perception Layer to Cognition Layer
//! - AgentAction: Output from Cognition Layer to Action Layer
//!
//! **Feature: phase3-ai-agent-evolution**
//! **Requirements: 1.2, 2.2**

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Message Context
// ============================================================================

/// Context information for a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContext {
    /// The message content
    pub content: String,
    /// Sender's user ID
    pub sender_id: Uuid,
    /// Sender's username (for display)
    pub sender_name: String,
    /// Group ID where the message was sent
    pub group_id: Uuid,
    /// Message ID
    pub message_id: Uuid,
    /// Timestamp when the message was sent
    pub timestamp: DateTime<Utc>,
    /// Whether this is a reply to the agent
    pub is_reply_to_agent: bool,
    /// Whether the agent was mentioned (@agent_name)
    pub mentions_agent: bool,
}

impl MessageContext {
    /// Create a new message context
    pub fn new(
        content: impl Into<String>,
        sender_id: Uuid,
        sender_name: impl Into<String>,
        group_id: Uuid,
    ) -> Self {
        Self {
            content: content.into(),
            sender_id,
            sender_name: sender_name.into(),
            group_id,
            message_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            is_reply_to_agent: false,
            mentions_agent: false,
        }
    }

    /// Set whether this is a reply to the agent
    pub fn with_reply_to_agent(mut self, is_reply: bool) -> Self {
        self.is_reply_to_agent = is_reply;
        self
    }

    /// Set whether the agent was mentioned
    pub fn with_mentions_agent(mut self, mentions: bool) -> Self {
        self.mentions_agent = mentions;
        self
    }

    /// Set the message ID
    pub fn with_message_id(mut self, id: Uuid) -> Self {
        self.message_id = id;
        self
    }

    /// Set the timestamp
    pub fn with_timestamp(mut self, ts: DateTime<Utc>) -> Self {
        self.timestamp = ts;
        self
    }
}

impl Default for MessageContext {
    fn default() -> Self {
        Self {
            content: String::new(),
            sender_id: Uuid::nil(),
            sender_name: String::new(),
            group_id: Uuid::nil(),
            message_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            is_reply_to_agent: false,
            mentions_agent: false,
        }
    }
}

// ============================================================================
// System Signals
// ============================================================================

/// System signal types for proactive agent behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    /// Group has been silent for a configured duration
    SilenceDetected {
        /// Duration of silence in seconds
        silence_duration_secs: u64,
    },
    /// Scheduled event (e.g., daily greeting)
    ScheduledEvent {
        /// Event type identifier
        event_type: ScheduledEventType,
    },
    /// Group activity pattern detected
    ActivityPattern {
        /// Pattern type
        pattern: ActivityPatternType,
    },
}

/// Types of scheduled events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduledEventType {
    /// Morning greeting
    MorningGreeting,
    /// Evening summary
    EveningSummary,
    /// Weekly recap
    WeeklyRecap,
    /// Custom scheduled event
    Custom,
}

/// Types of activity patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityPatternType {
    /// High activity period starting
    HighActivityStart,
    /// Activity winding down
    ActivityWindingDown,
    /// Unusual activity spike
    UnusualSpike,
}

// ============================================================================
// Feedback Types
// ============================================================================

/// User feedback types for learning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackType {
    /// Positive feedback (thumbs up, "thanks", etc.)
    Positive,
    /// Negative feedback (thumbs down, "wrong", etc.)
    Negative,
    /// User provided a correction
    Correction,
}

impl std::fmt::Display for FeedbackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeedbackType::Positive => write!(f, "positive"),
            FeedbackType::Negative => write!(f, "negative"),
            FeedbackType::Correction => write!(f, "correction"),
        }
    }
}

// ============================================================================
// Emotion State
// ============================================================================

/// Emotion state for expression variation
/// 
/// The agent's emotional state influences response style, language choice,
/// and emoji/sticker usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EmotionState {
    /// Neutral, balanced tone
    #[default]
    Neutral,
    /// Happy, positive mood
    Happy,
    /// Excited, enthusiastic
    Excited,
    /// Empathetic, understanding
    Empathetic,
    /// Supportive, encouraging
    Supportive,
    /// Curious, interested
    Curious,
    /// Playful, humorous
    Playful,
}

impl EmotionState {
    /// Get emoji suggestions for this emotion state
    pub fn suggested_emojis(&self) -> &[&str] {
        match self {
            EmotionState::Neutral => &["😊", "👍", "🙂"],
            EmotionState::Happy => &["😄", "🎉", "✨", "💫"],
            EmotionState::Excited => &["🎊", "🔥", "⚡", "🚀", "💪"],
            EmotionState::Empathetic => &["🤗", "💙", "🫂", "❤️"],
            EmotionState::Supportive => &["💪", "👏", "🌟", "✊"],
            EmotionState::Curious => &["🤔", "💭", "🧐", "❓"],
            EmotionState::Playful => &["😜", "😂", "🤣", "😏", "🎮"],
        }
    }

    /// Get language style hints for this emotion state
    pub fn style_hints(&self) -> &str {
        match self {
            EmotionState::Neutral => "balanced, informative",
            EmotionState::Happy => "cheerful, positive, use exclamations",
            EmotionState::Excited => "enthusiastic, energetic, use caps sparingly",
            EmotionState::Empathetic => "understanding, gentle, acknowledge feelings",
            EmotionState::Supportive => "encouraging, affirming, offer help",
            EmotionState::Curious => "questioning, interested, ask follow-ups",
            EmotionState::Playful => "humorous, light-hearted, use wordplay",
        }
    }
}

impl std::fmt::Display for EmotionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmotionState::Neutral => write!(f, "neutral"),
            EmotionState::Happy => write!(f, "happy"),
            EmotionState::Excited => write!(f, "excited"),
            EmotionState::Empathetic => write!(f, "empathetic"),
            EmotionState::Supportive => write!(f, "supportive"),
            EmotionState::Curious => write!(f, "curious"),
            EmotionState::Playful => write!(f, "playful"),
        }
    }
}


// ============================================================================
// Agent Event (Input from Perception Layer)
// ============================================================================

/// Multi-modal input event from the Perception Layer
///
/// AgentEvent represents all possible inputs that the agent can receive.
/// The Perception Layer is responsible for converting raw inputs (text, images,
/// audio) into these structured events.
///
/// **Feature: phase3-ai-agent-evolution, Property 1: Perception Layer Input Transformation**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    /// User sent a text message
    UserMessage(MessageContext),

    /// User sent an image (with optional caption)
    UserImage {
        /// URL of the image
        image_url: String,
        /// Optional caption/text accompanying the image
        caption: Option<String>,
        /// Message context
        context: MessageContext,
    },

    /// User sent a voice message (transcribed by STT)
    UserVoice {
        /// Transcribed text from the audio
        transcription: String,
        /// URL of the original audio file
        audio_url: String,
        /// Message context
        context: MessageContext,
    },

    /// System signal (silence detected, scheduled event, etc.)
    SystemSignal {
        /// Type of system signal
        signal_type: SignalType,
        /// Group ID this signal relates to
        group_id: Uuid,
    },

    /// Tool execution completed
    ToolOutput {
        /// Name of the tool that was executed
        tool_name: String,
        /// Result of the tool execution
        result: String,
        /// Request ID to correlate with the original CallTool action
        request_id: String,
        /// Whether the execution was successful
        success: bool,
    },

    /// User provided feedback on a response
    UserFeedback {
        /// ID of the message being rated
        message_id: Uuid,
        /// ID of the agent's response being rated
        response_id: Uuid,
        /// Type of feedback
        feedback_type: FeedbackType,
        /// Optional correction text (for FeedbackType::Correction)
        correction: Option<String>,
        /// User who provided the feedback
        user_id: Uuid,
    },
}

impl AgentEvent {
    /// Get the group ID associated with this event
    pub fn group_id(&self) -> Uuid {
        match self {
            AgentEvent::UserMessage(ctx) => ctx.group_id,
            AgentEvent::UserImage { context, .. } => context.group_id,
            AgentEvent::UserVoice { context, .. } => context.group_id,
            AgentEvent::SystemSignal { group_id, .. } => *group_id,
            AgentEvent::ToolOutput { .. } => Uuid::nil(), // Tool outputs don't have a group
            AgentEvent::UserFeedback { .. } => Uuid::nil(), // Feedback doesn't have a group
        }
    }

    /// Get the content/text of this event for processing
    pub fn content(&self) -> String {
        match self {
            AgentEvent::UserMessage(ctx) => ctx.content.clone(),
            AgentEvent::UserImage { caption, context, .. } => {
                caption.clone().unwrap_or_else(|| context.content.clone())
            }
            AgentEvent::UserVoice { transcription, .. } => transcription.clone(),
            AgentEvent::SystemSignal { signal_type, .. } => format!("{:?}", signal_type),
            AgentEvent::ToolOutput { tool_name, result, .. } => {
                format!("Tool {} returned: {}", tool_name, result)
            }
            AgentEvent::UserFeedback { feedback_type, correction, .. } => {
                if let Some(corr) = correction {
                    format!("Feedback: {} - {}", feedback_type, corr)
                } else {
                    format!("Feedback: {}", feedback_type)
                }
            }
        }
    }

    /// Check if this event requires a response
    pub fn requires_response(&self) -> bool {
        match self {
            AgentEvent::UserMessage(ctx) => ctx.mentions_agent || ctx.is_reply_to_agent,
            AgentEvent::UserImage { context, .. } => context.mentions_agent || context.is_reply_to_agent,
            AgentEvent::UserVoice { context, .. } => context.mentions_agent || context.is_reply_to_agent,
            AgentEvent::SystemSignal { .. } => true, // System signals always need handling
            AgentEvent::ToolOutput { .. } => true, // Tool outputs need to be processed
            AgentEvent::UserFeedback { .. } => false, // Feedback doesn't need a response
        }
    }

    /// Get the sender ID if this event has one
    pub fn sender_id(&self) -> Option<Uuid> {
        match self {
            AgentEvent::UserMessage(ctx) => Some(ctx.sender_id),
            AgentEvent::UserImage { context, .. } => Some(context.sender_id),
            AgentEvent::UserVoice { context, .. } => Some(context.sender_id),
            AgentEvent::UserFeedback { user_id, .. } => Some(*user_id),
            _ => None,
        }
    }

    /// Check if this is a multi-modal event (contains image or audio)
    pub fn is_multimodal(&self) -> bool {
        matches!(self, AgentEvent::UserImage { .. } | AgentEvent::UserVoice { .. })
    }
}

// ============================================================================
// Agent Action (Output to Action Layer)
// ============================================================================

/// Agent output action to be executed by the Action Layer
///
/// AgentAction represents all possible outputs that the agent can produce.
/// The Action Layer is responsible for executing these actions (sending messages,
/// calling APIs, etc.).
///
/// **Feature: phase3-ai-agent-evolution, Property 2: AgentCore Process Returns Valid Actions**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentAction {
    /// Reply with text message
    ReplyText {
        /// The text content to send
        content: String,
        /// Emotion state for styling the response
        emotion: EmotionState,
    },

    /// Reply with audio (TTS)
    ReplyAudio {
        /// Text to convert to speech
        text: String,
        /// Optional voice ID override (uses persona default if None)
        voice_id: Option<String>,
    },

    /// Reply with an image
    ReplyImage {
        /// URL of the image to send
        url: String,
        /// Optional caption for the image
        caption: Option<String>,
    },

    /// Reply with a sticker
    ReplySticker {
        /// Sticker ID to send
        sticker_id: String,
    },

    /// Call an external tool
    CallTool {
        /// Name of the tool to call
        name: String,
        /// Arguments for the tool (JSON)
        args: serde_json::Value,
        /// Request ID for correlating with ToolOutput event
        request_id: String,
    },

    /// Generate a summary of recent messages
    Summarize {
        /// Number of messages to summarize
        message_count: usize,
    },

    /// Do nothing (decided not to respond)
    Ignore,

    /// Continue thinking (chain of thought, internal)
    Think {
        /// The thought content (for logging/debugging)
        thought: String,
    },
}

impl AgentAction {
    /// Create a text reply action
    pub fn text(content: impl Into<String>) -> Self {
        AgentAction::ReplyText {
            content: content.into(),
            emotion: EmotionState::Neutral,
        }
    }

    /// Create a text reply with emotion
    pub fn text_with_emotion(content: impl Into<String>, emotion: EmotionState) -> Self {
        AgentAction::ReplyText {
            content: content.into(),
            emotion,
        }
    }

    /// Create an audio reply action
    pub fn audio(text: impl Into<String>) -> Self {
        AgentAction::ReplyAudio {
            text: text.into(),
            voice_id: None,
        }
    }

    /// Create a tool call action
    pub fn call_tool(name: impl Into<String>, args: serde_json::Value) -> Self {
        AgentAction::CallTool {
            name: name.into(),
            args,
            request_id: Uuid::new_v4().to_string(),
        }
    }

    /// Create an ignore action
    pub fn ignore() -> Self {
        AgentAction::Ignore
    }

    /// Create a think action
    pub fn think(thought: impl Into<String>) -> Self {
        AgentAction::Think {
            thought: thought.into(),
        }
    }

    /// Check if this action produces visible output
    pub fn is_visible(&self) -> bool {
        matches!(
            self,
            AgentAction::ReplyText { .. }
                | AgentAction::ReplyAudio { .. }
                | AgentAction::ReplyImage { .. }
                | AgentAction::ReplySticker { .. }
        )
    }

    /// Check if this action requires external API call
    pub fn requires_external_call(&self) -> bool {
        matches!(
            self,
            AgentAction::ReplyAudio { .. }
                | AgentAction::CallTool { .. }
                | AgentAction::Summarize { .. }
        )
    }

    /// Get the text content if this is a text-based action
    pub fn text_content(&self) -> Option<&str> {
        match self {
            AgentAction::ReplyText { content, .. } => Some(content),
            AgentAction::ReplyAudio { text, .. } => Some(text),
            AgentAction::ReplyImage { caption, .. } => caption.as_deref(),
            AgentAction::Think { thought } => Some(thought),
            _ => None,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_context_creation() {
        let ctx = MessageContext::new(
            "Hello, world!",
            Uuid::new_v4(),
            "test_user",
            Uuid::new_v4(),
        );

        assert_eq!(ctx.content, "Hello, world!");
        assert_eq!(ctx.sender_name, "test_user");
        assert!(!ctx.is_reply_to_agent);
        assert!(!ctx.mentions_agent);
    }

    #[test]
    fn test_message_context_builder() {
        let ctx = MessageContext::new("Test", Uuid::new_v4(), "user", Uuid::new_v4())
            .with_reply_to_agent(true)
            .with_mentions_agent(true);

        assert!(ctx.is_reply_to_agent);
        assert!(ctx.mentions_agent);
    }

    #[test]
    fn test_agent_event_group_id() {
        let group_id = Uuid::new_v4();
        let ctx = MessageContext::new("Test", Uuid::new_v4(), "user", group_id);
        let event = AgentEvent::UserMessage(ctx);

        assert_eq!(event.group_id(), group_id);
    }

    #[test]
    fn test_agent_event_content() {
        let ctx = MessageContext::new("Hello!", Uuid::new_v4(), "user", Uuid::new_v4());
        let event = AgentEvent::UserMessage(ctx);

        assert_eq!(event.content(), "Hello!");
    }

    #[test]
    fn test_agent_event_requires_response() {
        // Regular message doesn't require response
        let ctx = MessageContext::new("Hi", Uuid::new_v4(), "user", Uuid::new_v4());
        let event = AgentEvent::UserMessage(ctx);
        assert!(!event.requires_response());

        // Mention requires response
        let ctx = MessageContext::new("Hi", Uuid::new_v4(), "user", Uuid::new_v4())
            .with_mentions_agent(true);
        let event = AgentEvent::UserMessage(ctx);
        assert!(event.requires_response());

        // Reply requires response
        let ctx = MessageContext::new("Hi", Uuid::new_v4(), "user", Uuid::new_v4())
            .with_reply_to_agent(true);
        let event = AgentEvent::UserMessage(ctx);
        assert!(event.requires_response());
    }

    #[test]
    fn test_agent_event_is_multimodal() {
        let ctx = MessageContext::default();
        
        let text_event = AgentEvent::UserMessage(ctx.clone());
        assert!(!text_event.is_multimodal());

        let image_event = AgentEvent::UserImage {
            image_url: "http://example.com/img.jpg".to_string(),
            caption: None,
            context: ctx.clone(),
        };
        assert!(image_event.is_multimodal());

        let voice_event = AgentEvent::UserVoice {
            transcription: "Hello".to_string(),
            audio_url: "http://example.com/audio.mp3".to_string(),
            context: ctx,
        };
        assert!(voice_event.is_multimodal());
    }

    #[test]
    fn test_agent_action_creation() {
        let action = AgentAction::text("Hello!");
        assert!(matches!(action, AgentAction::ReplyText { content, emotion } 
            if content == "Hello!" && emotion == EmotionState::Neutral));

        let action = AgentAction::text_with_emotion("Yay!", EmotionState::Happy);
        assert!(matches!(action, AgentAction::ReplyText { emotion, .. } 
            if emotion == EmotionState::Happy));
    }

    #[test]
    fn test_agent_action_is_visible() {
        assert!(AgentAction::text("Hi").is_visible());
        assert!(AgentAction::audio("Hi").is_visible());
        assert!(!AgentAction::ignore().is_visible());
        assert!(!AgentAction::think("Hmm").is_visible());
    }

    #[test]
    fn test_agent_action_text_content() {
        let action = AgentAction::text("Hello");
        assert_eq!(action.text_content(), Some("Hello"));

        let action = AgentAction::ignore();
        assert_eq!(action.text_content(), None);
    }

    #[test]
    fn test_emotion_state_emojis() {
        let emojis = EmotionState::Happy.suggested_emojis();
        assert!(emojis.contains(&"😄"));

        let emojis = EmotionState::Empathetic.suggested_emojis();
        assert!(emojis.contains(&"🤗"));
    }

    #[test]
    fn test_emotion_state_display() {
        assert_eq!(EmotionState::Happy.to_string(), "happy");
        assert_eq!(EmotionState::Neutral.to_string(), "neutral");
    }

    #[test]
    fn test_feedback_type_display() {
        assert_eq!(FeedbackType::Positive.to_string(), "positive");
        assert_eq!(FeedbackType::Negative.to_string(), "negative");
        assert_eq!(FeedbackType::Correction.to_string(), "correction");
    }
}

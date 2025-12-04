//! Message data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Message type enum matching protobuf definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[repr(i16)]
pub enum MsgType {
    Text = 0,
    Image = 1,
    Video = 2,
    Emoji = 3,
}

impl From<i16> for MsgType {
    fn from(value: i16) -> Self {
        match value {
            0 => MsgType::Text,
            1 => MsgType::Image,
            2 => MsgType::Video,
            3 => MsgType::Emoji,
            _ => MsgType::Text,
        }
    }
}

impl From<i32> for MsgType {
    fn from(value: i32) -> Self {
        MsgType::from(value as i16)
    }
}

/// Chat message stored in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub msg_type: i16,
    pub content: String,
    pub timestamp: i64,
    pub is_ai_generated: bool,
    pub delivered: bool,
    pub created_at: DateTime<Utc>,
}

impl Message {
    /// Create a new private message
    pub fn new_private(
        sender_id: Uuid,
        receiver_id: Uuid,
        msg_type: MsgType,
        content: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender_id,
            receiver_id: Some(receiver_id),
            group_id: None,
            msg_type: msg_type as i16,
            content,
            timestamp: Utc::now().timestamp_millis(),
            is_ai_generated: false,
            delivered: false,
            created_at: Utc::now(),
        }
    }

    /// Create a new group message
    pub fn new_group(
        sender_id: Uuid,
        group_id: Uuid,
        msg_type: MsgType,
        content: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender_id,
            receiver_id: None,
            group_id: Some(group_id),
            msg_type: msg_type as i16,
            content,
            timestamp: Utc::now().timestamp_millis(),
            is_ai_generated: false,
            delivered: false,
            created_at: Utc::now(),
        }
    }

    /// Check if this is a private message
    pub fn is_private(&self) -> bool {
        self.receiver_id.is_some()
    }

    /// Check if this is a group message
    pub fn is_group(&self) -> bool {
        self.group_id.is_some()
    }

    /// Get the message type as enum
    pub fn get_msg_type(&self) -> MsgType {
        MsgType::from(self.msg_type)
    }
}

/// Message for creating a new message (input)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
    pub id: Option<Uuid>,
    pub sender_id: Uuid,
    pub receiver_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub msg_type: MsgType,
    pub content: String,
    pub is_ai_generated: bool,
}

/// Acknowledgment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AckStatus {
    Success = 0,
    Error = 1,
    Duplicate = 2,
}

/// Message acknowledgment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAck {
    pub message_id: Uuid,
    pub status: AckStatus,
    pub error_message: Option<String>,
}

impl MessageAck {
    pub fn success(message_id: Uuid) -> Self {
        Self {
            message_id,
            status: AckStatus::Success,
            error_message: None,
        }
    }

    pub fn error(message_id: Uuid, error: impl Into<String>) -> Self {
        Self {
            message_id,
            status: AckStatus::Error,
            error_message: Some(error.into()),
        }
    }

    pub fn duplicate(message_id: Uuid) -> Self {
        Self {
            message_id,
            status: AckStatus::Duplicate,
            error_message: Some("Duplicate message".to_string()),
        }
    }
}

/// Delivery receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReceipt {
    pub message_id: Uuid,
    pub delivered_to: Uuid,
    pub delivered_at: i64,
}

/// Offline message batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineMessageBatch {
    pub messages: Vec<Message>,
    pub has_more: bool,
    pub next_timestamp: Option<i64>,
}

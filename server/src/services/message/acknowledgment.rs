//! Acknowledgment System Module
//! 
//! Handles message acknowledgments and delivery receipts.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::models::{AckStatus, DeliveryReceipt, MessageAck};

/// Acknowledgment service for generating and handling acks
pub struct AcknowledgmentService;

impl AcknowledgmentService {
    /// Create a success acknowledgment
    pub fn success(message_id: Uuid) -> MessageAck {
        MessageAck::success(message_id)
    }

    /// Create an error acknowledgment
    pub fn error(message_id: Uuid, error: impl Into<String>) -> MessageAck {
        MessageAck::error(message_id, error)
    }

    /// Create a duplicate acknowledgment
    pub fn duplicate(message_id: Uuid) -> MessageAck {
        MessageAck::duplicate(message_id)
    }

    /// Create a delivery receipt
    pub fn delivery_receipt(message_id: Uuid, delivered_to: Uuid) -> DeliveryReceipt {
        DeliveryReceipt {
            message_id,
            delivered_to,
            delivered_at: chrono::Utc::now().timestamp_millis(),
        }
    }

    /// Serialize acknowledgment to JSON
    pub fn to_json(ack: &MessageAck) -> Result<String, serde_json::Error> {
        serde_json::to_string(ack)
    }

    /// Serialize delivery receipt to JSON
    pub fn receipt_to_json(receipt: &DeliveryReceipt) -> Result<String, serde_json::Error> {
        serde_json::to_string(receipt)
    }

    /// Parse acknowledgment from JSON
    pub fn from_json(json: &str) -> Result<MessageAck, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Server packet types for WebSocket communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerPacket {
    /// Chat message
    #[serde(rename = "message")]
    Message(super::models::Message),
    
    /// Message acknowledgment
    #[serde(rename = "ack")]
    Ack(MessageAck),
    
    /// Delivery receipt
    #[serde(rename = "receipt")]
    Receipt(DeliveryReceipt),
    
    /// Heartbeat response (pong)
    #[serde(rename = "pong")]
    Pong { timestamp: i64 },
    
    /// Error notification
    #[serde(rename = "error")]
    Error { code: String, message: String },
    
    /// Sync response with offline messages
    #[serde(rename = "sync")]
    Sync {
        messages: Vec<super::models::Message>,
        has_more: bool,
    },
}

impl ServerPacket {
    /// Create a pong response
    pub fn pong() -> Self {
        Self::Pong {
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    /// Create an error packet
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Client packet types for WebSocket communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientPacket {
    /// Chat message
    #[serde(rename = "message")]
    Message {
        id: Option<Uuid>,
        receiver_id: Option<Uuid>,
        group_id: Option<Uuid>,
        msg_type: i32,
        content: String,
    },
    
    /// Heartbeat (ping)
    #[serde(rename = "ping")]
    Ping { timestamp: i64 },
    
    /// Sync request for offline messages
    #[serde(rename = "sync")]
    Sync { since_timestamp: i64 },
    
    /// Typing indicator
    #[serde(rename = "typing")]
    Typing {
        receiver_id: Option<Uuid>,
        group_id: Option<Uuid>,
        is_typing: bool,
    },
}

impl ClientPacket {
    /// Parse from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_ack() {
        let message_id = Uuid::new_v4();
        let ack = AcknowledgmentService::success(message_id);
        
        assert_eq!(ack.message_id, message_id);
        assert_eq!(ack.status, AckStatus::Success);
        assert!(ack.error_message.is_none());
    }

    #[test]
    fn test_error_ack() {
        let message_id = Uuid::new_v4();
        let ack = AcknowledgmentService::error(message_id, "Test error");
        
        assert_eq!(ack.message_id, message_id);
        assert_eq!(ack.status, AckStatus::Error);
        assert_eq!(ack.error_message, Some("Test error".to_string()));
    }

    #[test]
    fn test_duplicate_ack() {
        let message_id = Uuid::new_v4();
        let ack = AcknowledgmentService::duplicate(message_id);
        
        assert_eq!(ack.message_id, message_id);
        assert_eq!(ack.status, AckStatus::Duplicate);
    }

    #[test]
    fn test_delivery_receipt() {
        let message_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let receipt = AcknowledgmentService::delivery_receipt(message_id, user_id);
        
        assert_eq!(receipt.message_id, message_id);
        assert_eq!(receipt.delivered_to, user_id);
        assert!(receipt.delivered_at > 0);
    }

    #[test]
    fn test_ack_serialization() {
        let message_id = Uuid::new_v4();
        let ack = AcknowledgmentService::success(message_id);
        
        let json = AcknowledgmentService::to_json(&ack).unwrap();
        let parsed = AcknowledgmentService::from_json(&json).unwrap();
        
        assert_eq!(parsed.message_id, ack.message_id);
        assert_eq!(parsed.status, ack.status);
    }

    #[test]
    fn test_server_packet_pong() {
        let packet = ServerPacket::pong();
        let json = packet.to_json().unwrap();
        
        assert!(json.contains("pong"));
        assert!(json.contains("timestamp"));
    }

    #[test]
    fn test_server_packet_error() {
        let packet = ServerPacket::error("AUTH_FAILED", "Invalid token");
        let json = packet.to_json().unwrap();
        
        assert!(json.contains("error"));
        assert!(json.contains("AUTH_FAILED"));
        assert!(json.contains("Invalid token"));
    }

    #[test]
    fn test_client_packet_parse() {
        let json = r#"{"type":"ping","timestamp":1234567890}"#;
        let packet = ClientPacket::from_json(json).unwrap();
        
        match packet {
            ClientPacket::Ping { timestamp } => {
                assert_eq!(timestamp, 1234567890);
            }
            _ => panic!("Expected Ping packet"),
        }
    }

    #[test]
    fn test_client_message_packet() {
        let json = r#"{"type":"message","id":null,"receiver_id":"550e8400-e29b-41d4-a716-446655440000","group_id":null,"msg_type":0,"content":"Hello"}"#;
        let packet = ClientPacket::from_json(json).unwrap();
        
        match packet {
            ClientPacket::Message { content, msg_type, .. } => {
                assert_eq!(content, "Hello");
                assert_eq!(msg_type, 0);
            }
            _ => panic!("Expected Message packet"),
        }
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property 24: Acknowledgment on successful persistence
        /// For any message ID, a success acknowledgment should be created correctly.
        #[test]
        fn prop_success_acknowledgment(
            message_id_bytes in prop::array::uniform16(any::<u8>()),
        ) {
            let message_id = Uuid::from_bytes(message_id_bytes);
            let ack = AcknowledgmentService::success(message_id);

            prop_assert_eq!(ack.message_id, message_id);
            prop_assert_eq!(ack.status, AckStatus::Success);
            prop_assert!(ack.error_message.is_none());
        }

        /// Property 25: Error acknowledgment on delivery failure
        /// For any message ID and error message, an error acknowledgment should be created.
        #[test]
        fn prop_error_acknowledgment(
            message_id_bytes in prop::array::uniform16(any::<u8>()),
            error_msg in "[a-zA-Z0-9 ]{1,100}",
        ) {
            let message_id = Uuid::from_bytes(message_id_bytes);
            let ack = AcknowledgmentService::error(message_id, error_msg.clone());

            prop_assert_eq!(ack.message_id, message_id);
            prop_assert_eq!(ack.status, AckStatus::Error);
            prop_assert_eq!(ack.error_message, Some(error_msg));
        }

        /// Property 26: Delivery receipt to sender
        /// For any message ID and user ID, a delivery receipt should be created.
        #[test]
        fn prop_delivery_receipt(
            message_id_bytes in prop::array::uniform16(any::<u8>()),
            user_id_bytes in prop::array::uniform16(any::<u8>()),
        ) {
            let message_id = Uuid::from_bytes(message_id_bytes);
            let user_id = Uuid::from_bytes(user_id_bytes);
            let receipt = AcknowledgmentService::delivery_receipt(message_id, user_id);

            prop_assert_eq!(receipt.message_id, message_id);
            prop_assert_eq!(receipt.delivered_to, user_id);
            prop_assert!(receipt.delivered_at > 0);
        }

        /// Property: Acknowledgment serialization round-trip
        #[test]
        fn prop_ack_serialization_roundtrip(
            message_id_bytes in prop::array::uniform16(any::<u8>()),
        ) {
            let message_id = Uuid::from_bytes(message_id_bytes);
            let ack = AcknowledgmentService::success(message_id);

            let json = AcknowledgmentService::to_json(&ack).unwrap();
            let parsed = AcknowledgmentService::from_json(&json).unwrap();

            prop_assert_eq!(parsed.message_id, ack.message_id);
            prop_assert_eq!(parsed.status, ack.status);
        }

        /// Property: Duplicate acknowledgment
        #[test]
        fn prop_duplicate_acknowledgment(
            message_id_bytes in prop::array::uniform16(any::<u8>()),
        ) {
            let message_id = Uuid::from_bytes(message_id_bytes);
            let ack = AcknowledgmentService::duplicate(message_id);

            prop_assert_eq!(ack.message_id, message_id);
            prop_assert_eq!(ack.status, AckStatus::Duplicate);
        }

        /// Property: Server packet serialization
        #[test]
        fn prop_server_packet_serialization(
            code in "[A-Z_]{1,20}",
            message in "[a-zA-Z0-9 ]{1,100}",
        ) {
            let packet = ServerPacket::error(code.clone(), message.clone());
            let json = packet.to_json().unwrap();

            prop_assert!(json.contains(&code));
            prop_assert!(json.contains(&message));
            prop_assert!(json.contains("error"));
        }

        /// Property: Pong packet has valid timestamp
        #[test]
        fn prop_pong_packet_timestamp(_seed in any::<u64>()) {
            let packet = ServerPacket::pong();
            
            match packet {
                ServerPacket::Pong { timestamp } => {
                    prop_assert!(timestamp > 0);
                }
                _ => prop_assert!(false, "Expected Pong packet"),
            }
        }
    }
}

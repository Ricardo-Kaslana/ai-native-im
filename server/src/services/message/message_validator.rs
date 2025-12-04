//! Message Validator Module
//! 
//! Validates message fields and content according to requirements.

use thiserror::Error;
use uuid::Uuid;

use super::models::{CreateMessage, MsgType};

/// Validation error types
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Message ID is required")]
    MissingMessageId,
    
    #[error("Sender ID is required")]
    MissingSenderId,
    
    #[error("Content is required")]
    MissingContent,
    
    #[error("Message must have either receiver_id or group_id, not both or neither")]
    InvalidTarget,
    
    #[error("Text content exceeds maximum length of {max} characters (got {actual})")]
    TextTooLong { max: usize, actual: usize },
    
    #[error("Invalid URL format for media message")]
    InvalidMediaUrl,
    
    #[error("Invalid emoji identifier")]
    InvalidEmoji,
    
    #[error("Invalid message type")]
    InvalidMessageType,
}

/// Maximum text message length (10000 characters)
pub const MAX_TEXT_LENGTH: usize = 10000;

/// Message validator
pub struct MessageValidator;

impl MessageValidator {
    /// Validate a message for creation
    pub fn validate(message: &CreateMessage) -> Result<(), ValidationError> {
        // Validate sender ID
        if message.sender_id == Uuid::nil() {
            return Err(ValidationError::MissingSenderId);
        }

        // Validate target (must have exactly one of receiver_id or group_id)
        match (&message.receiver_id, &message.group_id) {
            (Some(_), None) | (None, Some(_)) => {}
            _ => return Err(ValidationError::InvalidTarget),
        }

        // Validate content is not empty
        if message.content.trim().is_empty() {
            return Err(ValidationError::MissingContent);
        }

        // Validate based on message type
        match message.msg_type {
            MsgType::Text => Self::validate_text(&message.content)?,
            MsgType::Image | MsgType::Video => Self::validate_media_url(&message.content)?,
            MsgType::Emoji => Self::validate_emoji(&message.content)?,
        }

        Ok(())
    }

    /// Validate text message content
    fn validate_text(content: &str) -> Result<(), ValidationError> {
        let char_count = content.chars().count();
        if char_count > MAX_TEXT_LENGTH {
            return Err(ValidationError::TextTooLong {
                max: MAX_TEXT_LENGTH,
                actual: char_count,
            });
        }
        Ok(())
    }

    /// Validate media URL
    fn validate_media_url(url: &str) -> Result<(), ValidationError> {
        let url = url.trim();
        
        // Check if URL starts with http:// or https://
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ValidationError::InvalidMediaUrl);
        }

        // Basic URL structure validation
        if url.len() < 10 || !url.contains('.') {
            return Err(ValidationError::InvalidMediaUrl);
        }

        Ok(())
    }

    /// Validate emoji identifier
    fn validate_emoji(emoji: &str) -> Result<(), ValidationError> {
        let emoji = emoji.trim();
        
        // Check if it's a valid emoji (either Unicode emoji or custom emoji ID)
        if emoji.is_empty() {
            return Err(ValidationError::InvalidEmoji);
        }

        // Accept Unicode emojis (at least one character that's not ASCII)
        let has_emoji = emoji.chars().any(|c| !c.is_ascii() || c == ':');
        
        // Accept custom emoji format like :emoji_name: or emoji IDs
        let is_custom_emoji = emoji.starts_with(':') && emoji.ends_with(':') && emoji.len() > 2;
        let is_emoji_id = emoji.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-');

        if !has_emoji && !is_custom_emoji && !is_emoji_id {
            return Err(ValidationError::InvalidEmoji);
        }

        Ok(())
    }

    /// Validate message ID format
    pub fn validate_message_id(id: &str) -> Result<Uuid, ValidationError> {
        Uuid::parse_str(id).map_err(|_| ValidationError::MissingMessageId)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_private_message() -> CreateMessage {
        CreateMessage {
            id: Some(Uuid::new_v4()),
            sender_id: Uuid::new_v4(),
            receiver_id: Some(Uuid::new_v4()),
            group_id: None,
            msg_type: MsgType::Text,
            content: "Hello, world!".to_string(),
            is_ai_generated: false,
        }
    }

    fn create_valid_group_message() -> CreateMessage {
        CreateMessage {
            id: Some(Uuid::new_v4()),
            sender_id: Uuid::new_v4(),
            receiver_id: None,
            group_id: Some(Uuid::new_v4()),
            msg_type: MsgType::Text,
            content: "Hello, group!".to_string(),
            is_ai_generated: false,
        }
    }

    #[test]
    fn test_valid_private_message() {
        let message = create_valid_private_message();
        assert!(MessageValidator::validate(&message).is_ok());
    }

    #[test]
    fn test_valid_group_message() {
        let message = create_valid_group_message();
        assert!(MessageValidator::validate(&message).is_ok());
    }

    #[test]
    fn test_missing_sender_id() {
        let mut message = create_valid_private_message();
        message.sender_id = Uuid::nil();
        
        let result = MessageValidator::validate(&message);
        assert!(matches!(result, Err(ValidationError::MissingSenderId)));
    }

    #[test]
    fn test_invalid_target_both() {
        let mut message = create_valid_private_message();
        message.group_id = Some(Uuid::new_v4());
        
        let result = MessageValidator::validate(&message);
        assert!(matches!(result, Err(ValidationError::InvalidTarget)));
    }

    #[test]
    fn test_invalid_target_neither() {
        let mut message = create_valid_private_message();
        message.receiver_id = None;
        
        let result = MessageValidator::validate(&message);
        assert!(matches!(result, Err(ValidationError::InvalidTarget)));
    }

    #[test]
    fn test_empty_content() {
        let mut message = create_valid_private_message();
        message.content = "   ".to_string();
        
        let result = MessageValidator::validate(&message);
        assert!(matches!(result, Err(ValidationError::MissingContent)));
    }

    #[test]
    fn test_text_too_long() {
        let mut message = create_valid_private_message();
        message.content = "a".repeat(MAX_TEXT_LENGTH + 1);
        
        let result = MessageValidator::validate(&message);
        assert!(matches!(result, Err(ValidationError::TextTooLong { .. })));
    }

    #[test]
    fn test_text_at_max_length() {
        let mut message = create_valid_private_message();
        message.content = "a".repeat(MAX_TEXT_LENGTH);
        
        assert!(MessageValidator::validate(&message).is_ok());
    }

    #[test]
    fn test_valid_image_url() {
        let mut message = create_valid_private_message();
        message.msg_type = MsgType::Image;
        message.content = "https://example.com/image.jpg".to_string();
        
        assert!(MessageValidator::validate(&message).is_ok());
    }

    #[test]
    fn test_invalid_image_url() {
        let mut message = create_valid_private_message();
        message.msg_type = MsgType::Image;
        message.content = "not-a-url".to_string();
        
        let result = MessageValidator::validate(&message);
        assert!(matches!(result, Err(ValidationError::InvalidMediaUrl)));
    }

    #[test]
    fn test_valid_video_url() {
        let mut message = create_valid_private_message();
        message.msg_type = MsgType::Video;
        message.content = "http://example.com/video.mp4".to_string();
        
        assert!(MessageValidator::validate(&message).is_ok());
    }

    #[test]
    fn test_valid_unicode_emoji() {
        let mut message = create_valid_private_message();
        message.msg_type = MsgType::Emoji;
        message.content = "😀".to_string();
        
        assert!(MessageValidator::validate(&message).is_ok());
    }

    #[test]
    fn test_valid_custom_emoji() {
        let mut message = create_valid_private_message();
        message.msg_type = MsgType::Emoji;
        message.content = ":smile:".to_string();
        
        assert!(MessageValidator::validate(&message).is_ok());
    }

    #[test]
    fn test_valid_emoji_id() {
        let mut message = create_valid_private_message();
        message.msg_type = MsgType::Emoji;
        message.content = "emoji_123".to_string();
        
        assert!(MessageValidator::validate(&message).is_ok());
    }

    #[test]
    fn test_invalid_emoji() {
        let mut message = create_valid_private_message();
        message.msg_type = MsgType::Emoji;
        message.content = "".to_string();
        
        let result = MessageValidator::validate(&message);
        assert!(matches!(result, Err(ValidationError::MissingContent)));
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    fn valid_uuid() -> impl Strategy<Value = Uuid> {
        prop::array::uniform16(any::<u8>()).prop_map(Uuid::from_bytes)
    }

    fn non_nil_uuid() -> impl Strategy<Value = Uuid> {
        valid_uuid().prop_filter("UUID must not be nil", |u| *u != Uuid::nil())
    }

    proptest! {
        /// Property 7: Message validation completeness
        /// All required fields must be validated.
        #[test]
        fn prop_message_validation_completeness(
            sender_id in non_nil_uuid(),
            receiver_id in non_nil_uuid(),
            content in "[a-zA-Z0-9]{1,100}",  // Use alphanumeric characters only
        ) {
            let message = CreateMessage {
                id: Some(Uuid::new_v4()),
                sender_id,
                receiver_id: Some(receiver_id),
                group_id: None,
                msg_type: MsgType::Text,
                content,
                is_ai_generated: false,
            };

            // Valid message should pass validation
            prop_assert!(MessageValidator::validate(&message).is_ok());
        }

        /// Property 33: Text message length validation
        /// Text messages exceeding MAX_TEXT_LENGTH should be rejected.
        #[test]
        fn prop_text_length_validation(
            sender_id in non_nil_uuid(),
            receiver_id in non_nil_uuid(),
            extra_chars in 1usize..1000,
        ) {
            let content = "a".repeat(MAX_TEXT_LENGTH + extra_chars);
            let message = CreateMessage {
                id: Some(Uuid::new_v4()),
                sender_id,
                receiver_id: Some(receiver_id),
                group_id: None,
                msg_type: MsgType::Text,
                content,
                is_ai_generated: false,
            };

            let result = MessageValidator::validate(&message);
            let is_text_too_long = matches!(result, Err(ValidationError::TextTooLong { .. }));
            prop_assert!(is_text_too_long, "Expected TextTooLong error");
        }

        /// Property: Text at or below max length should be valid
        #[test]
        fn prop_text_within_limit(
            sender_id in non_nil_uuid(),
            receiver_id in non_nil_uuid(),
            length in 1usize..=MAX_TEXT_LENGTH,
        ) {
            let content = "a".repeat(length);
            let message = CreateMessage {
                id: Some(Uuid::new_v4()),
                sender_id,
                receiver_id: Some(receiver_id),
                group_id: None,
                msg_type: MsgType::Text,
                content,
                is_ai_generated: false,
            };

            prop_assert!(MessageValidator::validate(&message).is_ok());
        }

        /// Property 34: Media message URL validation
        /// Media messages must have valid URLs.
        #[test]
        fn prop_media_url_validation_valid(
            sender_id in non_nil_uuid(),
            receiver_id in non_nil_uuid(),
            domain in "[a-z]{3,10}",
            path in "[a-z0-9/]{1,20}",
            ext in "(jpg|png|gif|mp4|webm)",
        ) {
            let url = format!("https://{}.com/{}.{}", domain, path, ext);
            let message = CreateMessage {
                id: Some(Uuid::new_v4()),
                sender_id,
                receiver_id: Some(receiver_id),
                group_id: None,
                msg_type: MsgType::Image,
                content: url,
                is_ai_generated: false,
            };

            prop_assert!(MessageValidator::validate(&message).is_ok());
        }

        /// Property: Invalid URLs should be rejected for media messages
        #[test]
        fn prop_media_url_validation_invalid(
            sender_id in non_nil_uuid(),
            receiver_id in non_nil_uuid(),
            invalid_url in "[a-z]{1,20}",
        ) {
            // Ensure it doesn't accidentally start with http
            prop_assume!(!invalid_url.starts_with("http"));
            
            let message = CreateMessage {
                id: Some(Uuid::new_v4()),
                sender_id,
                receiver_id: Some(receiver_id),
                group_id: None,
                msg_type: MsgType::Image,
                content: invalid_url,
                is_ai_generated: false,
            };

            let result = MessageValidator::validate(&message);
            let is_invalid_url = matches!(result, Err(ValidationError::InvalidMediaUrl));
            prop_assert!(is_invalid_url, "Expected InvalidMediaUrl error");
        }

        /// Property 35: Emoji message validation
        /// Valid emoji identifiers should be accepted.
        #[test]
        fn prop_emoji_validation_custom(
            sender_id in non_nil_uuid(),
            receiver_id in non_nil_uuid(),
            emoji_name in "[a-z_]{1,20}",
        ) {
            let content = format!(":{}:", emoji_name);
            let message = CreateMessage {
                id: Some(Uuid::new_v4()),
                sender_id,
                receiver_id: Some(receiver_id),
                group_id: None,
                msg_type: MsgType::Emoji,
                content,
                is_ai_generated: false,
            };

            prop_assert!(MessageValidator::validate(&message).is_ok());
        }

        /// Property 36: Validation error rejection
        /// Messages with nil sender_id should be rejected.
        #[test]
        fn prop_nil_sender_rejected(
            receiver_id in non_nil_uuid(),
            content in ".{1,100}",
        ) {
            let message = CreateMessage {
                id: Some(Uuid::new_v4()),
                sender_id: Uuid::nil(),
                receiver_id: Some(receiver_id),
                group_id: None,
                msg_type: MsgType::Text,
                content,
                is_ai_generated: false,
            };

            let result = MessageValidator::validate(&message);
            let is_missing_sender = matches!(result, Err(ValidationError::MissingSenderId));
            prop_assert!(is_missing_sender, "Expected MissingSenderId error");
        }

        /// Property: Invalid target (both receiver and group) should be rejected
        #[test]
        fn prop_invalid_target_both(
            sender_id in non_nil_uuid(),
            receiver_id in non_nil_uuid(),
            group_id in non_nil_uuid(),
            content in ".{1,100}",
        ) {
            let message = CreateMessage {
                id: Some(Uuid::new_v4()),
                sender_id,
                receiver_id: Some(receiver_id),
                group_id: Some(group_id),
                msg_type: MsgType::Text,
                content,
                is_ai_generated: false,
            };

            let result = MessageValidator::validate(&message);
            let is_invalid_target = matches!(result, Err(ValidationError::InvalidTarget));
            prop_assert!(is_invalid_target, "Expected InvalidTarget error");
        }

        /// Property: Invalid target (neither receiver nor group) should be rejected
        #[test]
        fn prop_invalid_target_neither(
            sender_id in non_nil_uuid(),
            content in ".{1,100}",
        ) {
            let message = CreateMessage {
                id: Some(Uuid::new_v4()),
                sender_id,
                receiver_id: None,
                group_id: None,
                msg_type: MsgType::Text,
                content,
                is_ai_generated: false,
            };

            let result = MessageValidator::validate(&message);
            let is_invalid_target = matches!(result, Err(ValidationError::InvalidTarget));
            prop_assert!(is_invalid_target, "Expected InvalidTarget error");
        }

        /// Property: Empty content should be rejected
        #[test]
        fn prop_empty_content_rejected(
            sender_id in non_nil_uuid(),
            receiver_id in non_nil_uuid(),
            whitespace in "[ \t\n\r]{0,10}",
        ) {
            let message = CreateMessage {
                id: Some(Uuid::new_v4()),
                sender_id,
                receiver_id: Some(receiver_id),
                group_id: None,
                msg_type: MsgType::Text,
                content: whitespace,
                is_ai_generated: false,
            };

            let result = MessageValidator::validate(&message);
            let is_missing_content = matches!(result, Err(ValidationError::MissingContent));
            prop_assert!(is_missing_content, "Expected MissingContent error");
        }

        /// **Feature: message-system, Property 32: Message type support**
        /// For any message, the system should accept and process message types: 
        /// TEXT, IMAGE, VIDEO, and EMOJI.
        /// **Validates: Requirements 8.1**
        #[test]
        fn prop_message_type_support(
            sender_id in non_nil_uuid(),
            receiver_id in non_nil_uuid(),
            msg_type_idx in 0usize..4,
        ) {
            let (msg_type, content) = match msg_type_idx {
                0 => (MsgType::Text, "Valid text message".to_string()),
                1 => (MsgType::Image, "https://example.com/image.jpg".to_string()),
                2 => (MsgType::Video, "https://example.com/video.mp4".to_string()),
                3 => (MsgType::Emoji, "😀".to_string()),
                _ => unreachable!(),
            };

            let message = CreateMessage {
                id: Some(Uuid::new_v4()),
                sender_id,
                receiver_id: Some(receiver_id),
                group_id: None,
                msg_type,
                content,
                is_ai_generated: false,
            };

            // All four message types should be accepted
            let result = MessageValidator::validate(&message);
            prop_assert!(
                result.is_ok(),
                "Message type {:?} should be accepted, but got error: {:?}",
                msg_type,
                result.err()
            );
        }
    }
}

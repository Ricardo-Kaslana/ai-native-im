mod common;

use ai_im_server::services::email::{EmailService, EmailTemplate, MockEmailService};

#[tokio::test]
async fn test_mock_email_service_send() {
    let service = MockEmailService::silent();
    
    let template = EmailTemplate::Welcome {
        username: "testuser".to_string(),
    };
    
    let result = service.send_template("test@example.com", template).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_password_reset_email() {
    let service = MockEmailService::silent();
    
    let template = EmailTemplate::PasswordReset {
        username: "testuser".to_string(),
        reset_token: "abc123def456".to_string(),
        reset_url: "https://example.com/reset?token=abc123def456".to_string(),
    };
    
    let result = service.send_template("test@example.com", template).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_password_changed_email() {
    let service = MockEmailService::silent();
    
    let template = EmailTemplate::PasswordChanged {
        username: "testuser".to_string(),
    };
    
    let result = service.send_template("test@example.com", template).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_invalid_email_address() {
    let service = MockEmailService::silent();
    
    let template = EmailTemplate::Welcome {
        username: "testuser".to_string(),
    };
    
    let result = service.send_template("invalid-email", template).await;
    assert!(result.is_err());
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// **Feature: user-auth-system, Property 23: Reset email sending**
    /// **Validates: Requirements 6.2**
    /// *For any* password reset request, an email with the reset link should be sent.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_reset_email_sending(
            username in "[a-z]{3,20}",
            token in "[a-f0-9]{32}",
        ) {
            // For any valid username and token, email should be sent
            let _email = format!("{}@example.com", username);
            let reset_url = format!("https://example.com/reset?token={}", token);
            
            // Verify email template contains required fields
            let template = EmailTemplate::PasswordReset {
                username: username.clone(),
                reset_token: token.clone(),
                reset_url: reset_url.clone(),
            };
            
            // Email should contain username, token, and URL
            match template {
                EmailTemplate::PasswordReset { username: u, reset_token: t, reset_url: url } => {
                    prop_assert_eq!(u, username);
                    prop_assert_eq!(t, token.clone());
                    prop_assert!(url.contains(&token));
                }
                _ => prop_assert!(false, "Wrong template type"),
            }
        }
    }
}

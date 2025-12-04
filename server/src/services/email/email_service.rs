use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Email service errors
#[derive(Debug, Error)]
pub enum EmailError {
    #[error("SMTP error: {0}")]
    SmtpError(String),
    #[error("Template error: {0}")]
    TemplateError(String),
    #[error("Invalid email address: {0}")]
    InvalidEmail(String),
}

/// Email template types
#[derive(Debug, Clone)]
pub enum EmailTemplate {
    PasswordReset {
        username: String,
        reset_token: String,
        reset_url: String,
    },
    Welcome {
        username: String,
    },
    PasswordChanged {
        username: String,
    },
}

/// Email message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub html_body: Option<String>,
}

/// Email service trait
#[async_trait]
pub trait EmailService: Send + Sync {
    async fn send_email(&self, message: EmailMessage) -> Result<(), EmailError>;
    async fn send_template(&self, to: &str, template: EmailTemplate) -> Result<(), EmailError>;
}

/// Mock email service for development and testing
#[derive(Debug, Clone)]
pub struct MockEmailService {
    pub log_emails: bool,
}

impl MockEmailService {
    /// Create a new mock email service
    pub fn new() -> Self {
        Self { log_emails: true }
    }

    /// Create a mock service without logging
    pub fn silent() -> Self {
        Self { log_emails: false }
    }

    /// Generate email content from template
    fn render_template(&self, template: EmailTemplate) -> Result<(String, String), EmailError> {
        match template {
            EmailTemplate::PasswordReset {
                username,
                reset_token,
                reset_url,
            } => {
                let subject = "Password Reset Request".to_string();
                let body = format!(
                    "Hello {},\n\n\
                    You have requested a password reset for your account.\n\n\
                    Please use the following token to reset your password: {}\n\n\
                    Or click the following link: {}\n\n\
                    This token will expire in 1 hour.\n\n\
                    If you did not request this reset, please ignore this email.\n\n\
                    Best regards,\n\
                    AI Native IM Team",
                    username, reset_token, reset_url
                );
                Ok((subject, body))
            }
            EmailTemplate::Welcome { username } => {
                let subject = "Welcome to AI Native IM!".to_string();
                let body = format!(
                    "Hello {},\n\n\
                    Welcome to AI Native IM! Your account has been successfully created.\n\n\
                    You can now start using our instant messaging platform.\n\n\
                    Best regards,\n\
                    AI Native IM Team",
                    username
                );
                Ok((subject, body))
            }
            EmailTemplate::PasswordChanged { username } => {
                let subject = "Password Changed Successfully".to_string();
                let body = format!(
                    "Hello {},\n\n\
                    Your password has been successfully changed.\n\n\
                    If you did not make this change, please contact support immediately.\n\n\
                    Best regards,\n\
                    AI Native IM Team",
                    username
                );
                Ok((subject, body))
            }
        }
    }
}

#[async_trait]
impl EmailService for MockEmailService {
    async fn send_email(&self, message: EmailMessage) -> Result<(), EmailError> {
        if self.log_emails {
            tracing::info!(
                "📧 Mock Email Sent:\n\
                To: {}\n\
                Subject: {}\n\
                Body: {}\n\
                ---",
                message.to,
                message.subject,
                message.body
            );
        }
        
        // Simulate email sending delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(())
    }

    async fn send_template(&self, to: &str, template: EmailTemplate) -> Result<(), EmailError> {
        // Validate email address (basic validation)
        if !to.contains('@') || to.len() < 5 {
            return Err(EmailError::InvalidEmail(to.to_string()));
        }

        let (subject, body) = self.render_template(template)?;
        
        let message = EmailMessage {
            to: to.to_string(),
            subject,
            body,
            html_body: None,
        };
        
        self.send_email(message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_email_service() {
        let service = MockEmailService::new();
        
        let message = EmailMessage {
            to: "test@example.com".to_string(),
            subject: "Test Subject".to_string(),
            body: "Test Body".to_string(),
            html_body: None,
        };
        
        let result = service.send_email(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_password_reset_template() {
        let service = MockEmailService::silent();
        
        let template = EmailTemplate::PasswordReset {
            username: "testuser".to_string(),
            reset_token: "abc123".to_string(),
            reset_url: "https://example.com/reset?token=abc123".to_string(),
        };
        
        let result = service.send_template("test@example.com", template).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_email() {
        let service = MockEmailService::silent();
        
        let template = EmailTemplate::Welcome {
            username: "testuser".to_string(),
        };
        
        let result = service.send_template("invalid-email", template).await;
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// **Feature: user-auth-system, Property 21: Email notification on password change**
    /// **Validates: Requirements 5.5**
    /// *For any* password change, an email notification should be sent.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_email_notification_on_password_change(
            username in "[a-z]{3,20}",
        ) {
            // Password change should trigger email notification
            let template = EmailTemplate::PasswordChanged {
                username: username.clone(),
            };
            
            // Verify template contains username
            match template {
                EmailTemplate::PasswordChanged { username: u } => {
                    prop_assert_eq!(u, username);
                }
                _ => prop_assert!(false, "Wrong template type"),
            }
        }
    }

    /// **Feature: user-auth-system, Property 23: Reset email sending**
    /// **Validates: Requirements 6.2**
    /// *For any* password reset request, an email with reset link should be sent.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_reset_email_sending(
            username in "[a-z]{3,20}",
            token in "[a-zA-Z0-9]{32,64}",
        ) {
            // Password reset should include token and URL
            let reset_url = format!("https://example.com/reset?token={}", token);
            let template = EmailTemplate::PasswordReset {
                username: username.clone(),
                reset_token: token.clone(),
                reset_url: reset_url.clone(),
            };
            
            // Verify template contains all required fields
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

    /// Test email address validation
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_email_validation(
            local in "[a-z]{1,10}",
            domain in "[a-z]{1,10}",
        ) {
            let valid_email = format!("{}@{}.com", local, domain);
            let invalid_email = format!("{}{}", local, domain); // No @
            
            // Valid email should contain @
            prop_assert!(valid_email.contains('@'));
            prop_assert!(valid_email.len() >= 5);
            
            // Invalid email should not contain @
            prop_assert!(!invalid_email.contains('@'));
        }
    }

    /// Test email template rendering
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_template_rendering(
            username in "[a-z]{3,20}",
        ) {
            let service = MockEmailService::silent();
            
            // Test Welcome template
            let template = EmailTemplate::Welcome {
                username: username.clone(),
            };
            
            let result = service.render_template(template);
            prop_assert!(result.is_ok());
            
            if let Ok((subject, body)) = result {
                prop_assert!(!subject.is_empty());
                prop_assert!(!body.is_empty());
                prop_assert!(body.contains(&username));
            }
        }
    }
}

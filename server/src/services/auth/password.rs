use bcrypt::{hash, verify};
use thiserror::Error;
use uuid::Uuid;

/// Password service errors
#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Password must be at least 8 characters long")]
    TooShort,
    #[error("Password must contain at least one uppercase letter")]
    NoUppercase,
    #[error("Password must contain at least one lowercase letter")]
    NoLowercase,
    #[error("Password must contain at least one digit")]
    NoDigit,
    #[error("Password must contain at least one special character")]
    NoSpecialChar,
    #[error("Password hashing failed: {0}")]
    HashingFailed(#[from] bcrypt::BcryptError),
}

/// Password service for hashing, verification, and validation
#[derive(Clone)]
pub struct PasswordService {
    cost: u32,
}

impl Default for PasswordService {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordService {
    /// Create a new password service with default cost factor (12)
    pub fn new() -> Self {
        Self { cost: 12 }
    }

    /// Create a new password service with custom cost factor
    pub fn with_cost(cost: u32) -> Self {
        Self { cost }
    }

    /// Hash a password using bcrypt
    pub fn hash_password(&self, password: &str) -> Result<String, PasswordError> {
        hash(password, self.cost).map_err(PasswordError::HashingFailed)
    }

    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, PasswordError> {
        verify(password, hash).map_err(PasswordError::HashingFailed)
    }

    /// Validate password strength
    /// Requirements:
    /// - At least 8 characters
    /// - At least one uppercase letter
    /// - At least one lowercase letter
    /// - At least one digit
    /// - At least one special character
    pub fn validate_strength(&self, password: &str) -> Result<(), PasswordError> {
        if password.len() < 8 {
            return Err(PasswordError::TooShort);
        }
        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(PasswordError::NoUppercase);
        }
        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(PasswordError::NoLowercase);
        }
        if !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(PasswordError::NoDigit);
        }
        if !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(PasswordError::NoSpecialChar);
        }
        Ok(())
    }

    /// Generate a secure random reset token
    pub fn generate_reset_token(&self) -> String {
        Uuid::new_v4().to_string().replace("-", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Unit tests
    #[test]
    fn test_hash_and_verify_password() {
        let service = PasswordService::with_cost(4);
        let password = "TestPassword123!";

        let hash = service.hash_password(password).unwrap();
        assert!(service.verify_password(password, &hash).unwrap());
        assert!(!service.verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_password_too_short() {
        let service = PasswordService::new();
        assert!(matches!(
            service.validate_strength("Short1!"),
            Err(PasswordError::TooShort)
        ));
    }

    #[test]
    fn test_password_no_uppercase() {
        let service = PasswordService::new();
        assert!(matches!(
            service.validate_strength("password123!"),
            Err(PasswordError::NoUppercase)
        ));
    }

    #[test]
    fn test_password_no_lowercase() {
        let service = PasswordService::new();
        assert!(matches!(
            service.validate_strength("PASSWORD123!"),
            Err(PasswordError::NoLowercase)
        ));
    }

    #[test]
    fn test_password_no_digit() {
        let service = PasswordService::new();
        assert!(matches!(
            service.validate_strength("Password!!"),
            Err(PasswordError::NoDigit)
        ));
    }

    #[test]
    fn test_password_no_special_char() {
        let service = PasswordService::new();
        assert!(matches!(
            service.validate_strength("Password123"),
            Err(PasswordError::NoSpecialChar)
        ));
    }

    #[test]
    fn test_valid_password() {
        let service = PasswordService::new();
        assert!(service.validate_strength("ValidPass123!").is_ok());
    }

    #[test]
    fn test_generate_reset_token() {
        let service = PasswordService::new();
        let token1 = service.generate_reset_token();
        let token2 = service.generate_reset_token();

        assert_eq!(token1.len(), 32);
        assert_ne!(token1, token2);
    }

    // Property-based tests

    /// **Feature: user-auth-system, Property 2: Password complexity validation**
    /// **Validates: Requirements 1.2, 5.2**
    /// *For any* password string, if it passes validation, it must contain
    /// at least 8 characters, one uppercase, one lowercase, one digit, and one special char.
    proptest! {
        #[test]
        fn prop_password_complexity_validation(password in "[A-Za-z0-9!@#$%^&*]{0,20}") {
            let service = PasswordService::new();
            let result = service.validate_strength(&password);

            if result.is_ok() {
                prop_assert!(password.len() >= 8);
                prop_assert!(password.chars().any(|c| c.is_uppercase()));
                prop_assert!(password.chars().any(|c| c.is_lowercase()));
                prop_assert!(password.chars().any(|c| c.is_ascii_digit()));
                prop_assert!(password.chars().any(|c| !c.is_alphanumeric()));
            }
        }
    }

    /// **Feature: user-auth-system, Property 3: Password hashing with bcrypt**
    /// **Validates: Requirements 1.3, 5.3**
    /// *For any* valid password, hashing produces a different string than the original.
    proptest! {
        #[test]
        fn prop_password_hashing_produces_different_output(password in ".{8,50}") {
            let service = PasswordService::with_cost(4);
            let hash = service.hash_password(&password).unwrap();

            prop_assert_ne!(password, hash.clone());
            prop_assert!(hash.starts_with("$2"));
        }
    }

    /// **Feature: user-auth-system, Property 7: Password verification with bcrypt**
    /// **Validates: Requirements 2.2**
    /// *For any* password, hashing then verifying with the same password returns true.
    proptest! {
        #[test]
        fn prop_password_verification_roundtrip(password in ".{1,50}") {
            let service = PasswordService::with_cost(4);
            let hash = service.hash_password(&password).unwrap();

            prop_assert!(service.verify_password(&password, &hash).unwrap());
        }
    }

    /// *For any* two different passwords, verifying one against the other's hash returns false.
    proptest! {
        #[test]
        fn prop_different_passwords_fail_verification(
            password1 in ".{8,30}",
            password2 in ".{8,30}"
        ) {
            prop_assume!(password1 != password2);

            let service = PasswordService::with_cost(4);
            let hash = service.hash_password(&password1).unwrap();

            prop_assert!(!service.verify_password(&password2, &hash).unwrap());
        }
    }
}

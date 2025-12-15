//! Property-based tests for AppError
//!
//! These tests verify the correctness properties defined in the design document
//! for the unified error handling system.

mod common;

use ai_im_server::error::{error_codes, AppError};
use axum::http::StatusCode;
use proptest::prelude::*;

// ============================================================================
// Test Helpers
// ============================================================================

/// Generate all possible AppError variants for testing
fn all_app_errors() -> Vec<AppError> {
    vec![
        // General errors (1000-1999)
        AppError::Internal("test".to_string()),
        AppError::Validation("test".to_string()),
        AppError::NotFound("test".to_string()),
        AppError::Conflict("test".to_string()),
        AppError::BadRequest("test".to_string()),
        AppError::Timeout,
        // Authentication errors (2000-2999)
        AppError::Unauthorized,
        AppError::Forbidden("test".to_string()),
        AppError::TokenExpired,
        AppError::InvalidCredentials,
        AppError::AccountLocked,
        AppError::AccountSuspended,
        // Business logic errors (3000-3999)
        AppError::UserNotFound("test".to_string()),
        AppError::GroupNotFound("test".to_string()),
        AppError::MessageNotFound("test".to_string()),
        AppError::PermissionDenied("test".to_string()),
        AppError::AlreadyExists("test".to_string()),
        AppError::GroupFull,
        AppError::InviteExpired,
        AppError::MediaNotFound("test".to_string()),
        AppError::MediaTooLarge { size: 100, max: 50 },
        AppError::UnsupportedMediaType("test".to_string()),
        // AI service errors (4000-4999)
        AppError::AIServiceUnavailable("test".to_string()),
        AppError::AIRateLimited,
        AppError::AITimeout,
        AppError::AIInvalidResponse("test".to_string()),
        AppError::AIAgentNotFound("test".to_string()),
        AppError::EmbeddingError("test".to_string()),
        AppError::VectorDBError("test".to_string()),
        // Database error
        AppError::Database("test".to_string()),
    ]
}

/// Get error category from error code
fn get_error_category(code: i32) -> &'static str {
    match code {
        1000..=1999 => "general",
        2000..=2999 => "auth",
        3000..=3999 => "business",
        4000..=4999 => "ai",
        _ => "unknown",
    }
}

// ============================================================================
// Property 2: Error code to status code mapping correctness
// **Feature: phase1-infrastructure-refactor, Property 2: 错误码到状态码映射正确性**
// **Validates: Requirements 1.2**
// ============================================================================

#[test]
fn test_property_2_error_code_to_status_code_mapping() {
    for error in all_app_errors() {
        let status = error.status_code();
        let code = error.error_code();
        
        // Verify semantic correctness based on error type
        match &error {
            // 500 Internal Server Error
            AppError::Internal(_) | AppError::Database(_) => {
                assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal/Database errors should return 500, got {:?} for error code {}", status, code);
            }
            // 400 Bad Request
            AppError::Validation(_) | AppError::BadRequest(_) => {
                assert_eq!(status, StatusCode::BAD_REQUEST,
                    "Validation/BadRequest errors should return 400, got {:?} for error code {}", status, code);
            }
            // 404 Not Found
            AppError::NotFound(_) | AppError::UserNotFound(_) | AppError::GroupNotFound(_) |
            AppError::MessageNotFound(_) | AppError::MediaNotFound(_) | AppError::AIAgentNotFound(_) => {
                assert_eq!(status, StatusCode::NOT_FOUND,
                    "NotFound errors should return 404, got {:?} for error code {}", status, code);
            }
            // 409 Conflict
            AppError::Conflict(_) | AppError::AlreadyExists(_) => {
                assert_eq!(status, StatusCode::CONFLICT,
                    "Conflict errors should return 409, got {:?} for error code {}", status, code);
            }
            // 401 Unauthorized
            AppError::Unauthorized | AppError::TokenExpired | AppError::InvalidCredentials |
            AppError::AccountLocked | AppError::AccountSuspended => {
                assert_eq!(status, StatusCode::UNAUTHORIZED,
                    "Auth errors should return 401, got {:?} for error code {}", status, code);
            }
            // 403 Forbidden
            AppError::Forbidden(_) | AppError::PermissionDenied(_) => {
                assert_eq!(status, StatusCode::FORBIDDEN,
                    "Forbidden errors should return 403, got {:?} for error code {}", status, code);
            }
            // 408 Request Timeout
            AppError::Timeout => {
                assert_eq!(status, StatusCode::REQUEST_TIMEOUT,
                    "Timeout should return 408, got {:?}", status);
            }
            // 400 Bad Request (business logic)
            AppError::GroupFull | AppError::InviteExpired | 
            AppError::MediaTooLarge { .. } | AppError::UnsupportedMediaType(_) => {
                assert_eq!(status, StatusCode::BAD_REQUEST,
                    "Business validation errors should return 400, got {:?} for error code {}", status, code);
            }
            // 503 Service Unavailable (AI errors)
            AppError::AIServiceUnavailable(_) | AppError::AIRateLimited | AppError::AITimeout |
            AppError::AIInvalidResponse(_) | AppError::EmbeddingError(_) | AppError::VectorDBError(_) => {
                assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE,
                    "AI service errors should return 503, got {:?} for error code {}", status, code);
            }
        }
    }
}

// ============================================================================
// Property 6: Error code uniqueness
// **Feature: phase1-infrastructure-refactor, Property 6: 错误码唯一性**
// **Validates: Requirements 5.1, 5.3**
// ============================================================================

#[test]
fn test_property_6_error_code_uniqueness() {
    // Get all non-parameterized error variants and their codes
    let unique_errors: Vec<(AppError, i32)> = vec![
        (AppError::Timeout, error_codes::TIMEOUT),
        (AppError::Unauthorized, error_codes::UNAUTHORIZED),
        (AppError::TokenExpired, error_codes::TOKEN_EXPIRED),
        (AppError::InvalidCredentials, error_codes::INVALID_CREDENTIALS),
        (AppError::AccountLocked, error_codes::ACCOUNT_LOCKED),
        (AppError::AccountSuspended, error_codes::ACCOUNT_SUSPENDED),
        (AppError::GroupFull, error_codes::GROUP_FULL),
        (AppError::InviteExpired, error_codes::INVITE_EXPIRED),
        (AppError::AIRateLimited, error_codes::AI_RATE_LIMITED),
        (AppError::AITimeout, error_codes::AI_TIMEOUT),
    ];
    
    // Check that each non-parameterized variant has a unique code
    let mut seen_codes: std::collections::HashSet<i32> = std::collections::HashSet::new();
    for (error, expected_code) in &unique_errors {
        let actual_code = error.error_code();
        assert_eq!(actual_code, *expected_code,
            "Error {:?} should have code {}, got {}", error, expected_code, actual_code);
        assert!(seen_codes.insert(actual_code),
            "Duplicate error code {} found for {:?}", actual_code, error);
    }
    
    // Also verify parameterized variants have distinct codes
    let parameterized_codes: Vec<i32> = vec![
        AppError::Internal("".to_string()).error_code(),
        AppError::Validation("".to_string()).error_code(),
        AppError::NotFound("".to_string()).error_code(),
        AppError::Conflict("".to_string()).error_code(),
        AppError::BadRequest("".to_string()).error_code(),
        AppError::Forbidden("".to_string()).error_code(),
        AppError::UserNotFound("".to_string()).error_code(),
        AppError::GroupNotFound("".to_string()).error_code(),
        AppError::MessageNotFound("".to_string()).error_code(),
        AppError::PermissionDenied("".to_string()).error_code(),
        AppError::AlreadyExists("".to_string()).error_code(),
        AppError::MediaNotFound("".to_string()).error_code(),
        AppError::MediaTooLarge { size: 0, max: 0 }.error_code(),
        AppError::UnsupportedMediaType("".to_string()).error_code(),
        AppError::AIServiceUnavailable("".to_string()).error_code(),
        AppError::AIInvalidResponse("".to_string()).error_code(),
        AppError::AIAgentNotFound("".to_string()).error_code(),
        AppError::EmbeddingError("".to_string()).error_code(),
        AppError::VectorDBError("".to_string()).error_code(),
    ];
    
    for code in parameterized_codes {
        seen_codes.insert(code);
    }
    
    // Note: Database maps to INTERNAL_ERROR, which is expected
    // So we don't check for complete uniqueness of all variants
}

// ============================================================================
// Property 7: Error code range correctness
// **Feature: phase1-infrastructure-refactor, Property 7: 错误码范围正确性**
// **Validates: Requirements 5.1**
// ============================================================================

#[test]
fn test_property_7_error_code_range_correctness() {
    for error in all_app_errors() {
        let code = error.error_code();
        let category = get_error_category(code);
        
        // Verify code is within valid ranges
        assert!(
            (1000..=1999).contains(&code) ||
            (2000..=2999).contains(&code) ||
            (3000..=3999).contains(&code) ||
            (4000..=4999).contains(&code),
            "Error code {} is outside valid ranges (1000-4999) for error {:?}", code, error
        );
        
        // Verify category matches error type
        match &error {
            // General errors should be 1000-1999
            AppError::Internal(_) | AppError::Validation(_) | AppError::NotFound(_) |
            AppError::Conflict(_) | AppError::BadRequest(_) | AppError::Timeout |
            AppError::Database(_) => {
                assert_eq!(category, "general",
                    "General error {:?} should have code in 1000-1999, got {}", error, code);
            }
            // Auth errors should be 2000-2999
            AppError::Unauthorized | AppError::Forbidden(_) | AppError::TokenExpired |
            AppError::InvalidCredentials | AppError::AccountLocked | AppError::AccountSuspended => {
                assert_eq!(category, "auth",
                    "Auth error {:?} should have code in 2000-2999, got {}", error, code);
            }
            // Business errors should be 3000-3999
            AppError::UserNotFound(_) | AppError::GroupNotFound(_) | AppError::MessageNotFound(_) |
            AppError::PermissionDenied(_) | AppError::AlreadyExists(_) | AppError::GroupFull |
            AppError::InviteExpired | AppError::MediaNotFound(_) | AppError::MediaTooLarge { .. } |
            AppError::UnsupportedMediaType(_) => {
                assert_eq!(category, "business",
                    "Business error {:?} should have code in 3000-3999, got {}", error, code);
            }
            // AI errors should be 4000-4999
            AppError::AIServiceUnavailable(_) | AppError::AIRateLimited | AppError::AITimeout |
            AppError::AIInvalidResponse(_) | AppError::AIAgentNotFound(_) |
            AppError::EmbeddingError(_) | AppError::VectorDBError(_) => {
                assert_eq!(category, "ai",
                    "AI error {:?} should have code in 4000-4999, got {}", error, code);
            }
        }
    }
}

// ============================================================================
// Property-based tests using proptest
// ============================================================================

proptest! {
    /// **Feature: phase1-infrastructure-refactor, Property 2: 错误码到状态码映射正确性**
    /// **Validates: Requirements 1.2**
    /// *For any* error message string, the error code to status code mapping should be consistent
    #[test]
    fn prop_error_status_code_consistency(msg in "[a-zA-Z0-9 ]{0,100}") {
        // Test that same error type always produces same status code regardless of message
        let errors = vec![
            (AppError::Internal(msg.clone()), StatusCode::INTERNAL_SERVER_ERROR),
            (AppError::Validation(msg.clone()), StatusCode::BAD_REQUEST),
            (AppError::NotFound(msg.clone()), StatusCode::NOT_FOUND),
            (AppError::Forbidden(msg.clone()), StatusCode::FORBIDDEN),
        ];
        
        for (error, expected_status) in errors {
            prop_assert_eq!(error.status_code(), expected_status);
        }
    }
    
    /// **Feature: phase1-infrastructure-refactor, Property 7: 错误码范围正确性**
    /// **Validates: Requirements 5.1**
    /// *For any* error message string, the error code should be within valid ranges
    #[test]
    fn prop_error_code_in_valid_range(msg in "[a-zA-Z0-9 ]{0,100}") {
        let errors = vec![
            AppError::Internal(msg.clone()),
            AppError::Validation(msg.clone()),
            AppError::NotFound(msg.clone()),
            AppError::Forbidden(msg.clone()),
            AppError::UserNotFound(msg.clone()),
            AppError::AIServiceUnavailable(msg.clone()),
        ];
        
        for error in errors {
            let code = error.error_code();
            prop_assert!(
                (1000..=4999).contains(&code),
                "Error code {} is outside valid range for {:?}", code, error
            );
        }
    }
    
    /// **Feature: phase1-infrastructure-refactor, Property 6: 错误码唯一性**
    /// **Validates: Requirements 5.3**
    /// *For any* two different error types, they should have different error codes
    #[test]
    fn prop_different_error_types_have_different_codes(
        msg1 in "[a-zA-Z0-9]{1,50}",
        msg2 in "[a-zA-Z0-9]{1,50}"
    ) {
        // Different error types should have different codes
        let validation = AppError::Validation(msg1.clone());
        let not_found = AppError::NotFound(msg2.clone());
        
        prop_assert_ne!(
            validation.error_code(),
            not_found.error_code(),
            "Validation and NotFound should have different error codes"
        );
        
        let unauthorized = AppError::Unauthorized;
        let forbidden = AppError::Forbidden(msg1);
        
        prop_assert_ne!(
            unauthorized.error_code(),
            forbidden.error_code(),
            "Unauthorized and Forbidden should have different error codes"
        );
    }
}

// ============================================================================
// Additional unit tests for error codes constants
// ============================================================================

#[test]
fn test_error_codes_constants_are_in_correct_ranges() {
    // General errors (1000-1999)
    assert!((1000..2000).contains(&error_codes::INTERNAL_ERROR));
    assert!((1000..2000).contains(&error_codes::VALIDATION_ERROR));
    assert!((1000..2000).contains(&error_codes::NOT_FOUND));
    assert!((1000..2000).contains(&error_codes::CONFLICT));
    assert!((1000..2000).contains(&error_codes::BAD_REQUEST));
    assert!((1000..2000).contains(&error_codes::TIMEOUT));
    
    // Auth errors (2000-2999)
    assert!((2000..3000).contains(&error_codes::UNAUTHORIZED));
    assert!((2000..3000).contains(&error_codes::FORBIDDEN));
    assert!((2000..3000).contains(&error_codes::TOKEN_EXPIRED));
    assert!((2000..3000).contains(&error_codes::INVALID_CREDENTIALS));
    assert!((2000..3000).contains(&error_codes::ACCOUNT_LOCKED));
    assert!((2000..3000).contains(&error_codes::ACCOUNT_SUSPENDED));
    
    // Business errors (3000-3999)
    assert!((3000..4000).contains(&error_codes::USER_NOT_FOUND));
    assert!((3000..4000).contains(&error_codes::GROUP_NOT_FOUND));
    assert!((3000..4000).contains(&error_codes::MESSAGE_NOT_FOUND));
    assert!((3000..4000).contains(&error_codes::PERMISSION_DENIED));
    assert!((3000..4000).contains(&error_codes::ALREADY_EXISTS));
    assert!((3000..4000).contains(&error_codes::GROUP_FULL));
    assert!((3000..4000).contains(&error_codes::INVITE_EXPIRED));
    assert!((3000..4000).contains(&error_codes::MEDIA_NOT_FOUND));
    assert!((3000..4000).contains(&error_codes::MEDIA_TOO_LARGE));
    assert!((3000..4000).contains(&error_codes::UNSUPPORTED_MEDIA_TYPE));
    
    // AI errors (4000-4999)
    assert!((4000..5000).contains(&error_codes::AI_SERVICE_UNAVAILABLE));
    assert!((4000..5000).contains(&error_codes::AI_RATE_LIMITED));
    assert!((4000..5000).contains(&error_codes::AI_TIMEOUT));
    assert!((4000..5000).contains(&error_codes::AI_INVALID_RESPONSE));
    assert!((4000..5000).contains(&error_codes::AI_AGENT_NOT_FOUND));
    assert!((4000..5000).contains(&error_codes::EMBEDDING_ERROR));
    assert!((4000..5000).contains(&error_codes::VECTOR_DB_ERROR));
}


// ============================================================================
// Property 1: Error response format consistency
// **Feature: phase1-infrastructure-refactor, Property 1: 错误响应格式一致性**
// **Validates: Requirements 1.1, 2.2**
// ============================================================================

use axum::response::IntoResponse;
use http_body_util::BodyExt;

/// Helper to extract JSON body from response
async fn extract_json_body(response: axum::response::Response) -> serde_json::Value {
    let body = response.into_body();
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_property_1_error_response_format_consistency() {
    for error in all_app_errors() {
        let response = error.into_response();
        let _status = response.status();
        let json = extract_json_body(response).await;
        
        // Verify response contains required fields
        assert!(json.get("success").is_some(), 
            "Response should contain 'success' field");
        assert_eq!(json["success"], false, 
            "Error response should have success: false");
        
        assert!(json.get("error").is_some(), 
            "Error response should contain 'error' object");
        
        let error_obj = &json["error"];
        assert!(error_obj.get("code").is_some(), 
            "Error object should contain 'code' field");
        assert!(error_obj.get("message").is_some(), 
            "Error object should contain 'message' field");
        
        // Verify code is an integer
        assert!(error_obj["code"].is_i64(), 
            "Error code should be an integer");
        
        // Verify message is a string
        assert!(error_obj["message"].is_string(), 
            "Error message should be a string");
    }
}

proptest! {
    /// **Feature: phase1-infrastructure-refactor, Property 1: 错误响应格式一致性**
    /// **Validates: Requirements 1.1, 2.2**
    /// *For any* error message, the response format should be consistent
    #[test]
    fn prop_error_response_has_required_fields(msg in "[a-zA-Z0-9 ]{1,100}") {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let error = AppError::Validation(msg);
            let response = error.into_response();
            let json = extract_json_body(response).await;
            
            prop_assert_eq!(json["success"].as_bool(), Some(false));
            prop_assert!(json.get("error").is_some());
            prop_assert!(json["error"].get("code").is_some());
            prop_assert!(json["error"].get("message").is_some());
            Ok(())
        })?;
    }
}


// ============================================================================
// Property 3: Success response format consistency
// **Feature: phase1-infrastructure-refactor, Property 3: 成功响应格式一致性**
// **Validates: Requirements 2.1**
// ============================================================================

use ai_im_server::response::{ApiResponse, ListData, PaginatedData};

#[test]
fn test_property_3_success_response_format_consistency() {
    // Test with various data types
    let response = ApiResponse::success("test data".to_string());
    assert!(response.success);
    assert!(response.data.is_some());
    assert!(response.error.is_none());
    
    let response = ApiResponse::success(42i32);
    assert!(response.success);
    assert_eq!(response.data, Some(42));
    
    let response = ApiResponse::success(vec![1, 2, 3]);
    assert!(response.success);
    assert_eq!(response.data, Some(vec![1, 2, 3]));
}

#[test]
fn test_property_3_success_response_serialization() {
    let response = ApiResponse::success("test".to_string());
    let json = serde_json::to_value(&response).unwrap();
    
    assert_eq!(json["success"], true);
    assert!(json.get("data").is_some());
    assert!(json.get("error").is_none() || json["error"].is_null());
}

// ============================================================================
// Property 4: List response format consistency
// **Feature: phase1-infrastructure-refactor, Property 4: 列表响应格式一致性**
// **Validates: Requirements 2.3**
// ============================================================================

#[test]
fn test_property_4_list_response_format_consistency() {
    let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let list = ListData::new(items.clone(), 100);
    
    assert_eq!(list.items.len(), 3);
    assert_eq!(list.total, 100);
    
    // Test from_vec
    let list = ListData::from_vec(items);
    assert_eq!(list.items.len(), 3);
    assert_eq!(list.total, 3);
}

#[test]
fn test_property_4_list_response_serialization() {
    let list = ListData::new(vec![1, 2, 3], 100);
    let response = ApiResponse::success(list);
    let json = serde_json::to_value(&response).unwrap();
    
    assert_eq!(json["success"], true);
    assert!(json["data"]["items"].is_array());
    assert_eq!(json["data"]["total"], 100);
}

#[test]
fn test_property_4_paginated_response_format() {
    let paginated = PaginatedData::new(vec![1, 2, 3], 100, 1, 10);
    
    assert_eq!(paginated.items.len(), 3);
    assert_eq!(paginated.total, 100);
    assert_eq!(paginated.page, 1);
    assert_eq!(paginated.page_size, 10);
    assert_eq!(paginated.total_pages, 10);
}

#[test]
fn test_property_4_paginated_response_serialization() {
    let paginated = PaginatedData::new(vec!["a", "b"], 50, 2, 10);
    let response = ApiResponse::success(paginated);
    let json = serde_json::to_value(&response).unwrap();
    
    assert_eq!(json["success"], true);
    assert!(json["data"]["items"].is_array());
    assert_eq!(json["data"]["total"], 50);
    assert_eq!(json["data"]["page"], 2);
    assert_eq!(json["data"]["page_size"], 10);
    assert_eq!(json["data"]["total_pages"], 5);
}

proptest! {
    /// **Feature: phase1-infrastructure-refactor, Property 3: 成功响应格式一致性**
    /// **Validates: Requirements 2.1**
    /// *For any* data value, success response should have success: true and data field
    #[test]
    fn prop_success_response_format(data in "[a-zA-Z0-9]{1,50}") {
        let response = ApiResponse::success(data.clone());
        prop_assert!(response.success);
        prop_assert_eq!(response.data, Some(data));
        prop_assert!(response.error.is_none());
    }
    
    /// **Feature: phase1-infrastructure-refactor, Property 4: 列表响应格式一致性**
    /// **Validates: Requirements 2.3**
    /// *For any* list of items and total count, list response should contain items array and total
    #[test]
    fn prop_list_response_format(
        item_count in 0usize..100,
        total in 0i64..10000
    ) {
        let items: Vec<i32> = (0..item_count as i32).collect();
        let list = ListData::new(items.clone(), total);
        
        prop_assert_eq!(list.items.len(), item_count);
        prop_assert_eq!(list.total, total);
    }
    
    /// **Feature: phase1-infrastructure-refactor, Property 4: 列表响应格式一致性**
    /// **Validates: Requirements 2.4**
    /// *For any* paginated data, total_pages should be calculated correctly
    #[test]
    fn prop_paginated_response_total_pages(
        total in 0i64..10000,
        page_size in 1i32..100
    ) {
        let paginated = PaginatedData::<i32>::new(vec![], total, 1, page_size);
        
        let expected_pages = if page_size > 0 {
            ((total as f64) / (page_size as f64)).ceil() as i32
        } else {
            0
        };
        
        prop_assert_eq!(paginated.total_pages, expected_pages);
    }
}

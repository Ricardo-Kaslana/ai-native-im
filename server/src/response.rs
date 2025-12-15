//! Unified API Response Module
//!
//! This module provides standardized response structures for all API endpoints.
//! All successful responses should use `ApiResponse<T>` to ensure consistent
//! response format across the application.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// Unified API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

/// Error information for failed responses
#[derive(Debug, Serialize)]
pub struct ErrorInfo {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a successful response with data
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create a successful response without data (for operations like delete)
    pub fn ok() -> ApiResponse<()> {
        ApiResponse {
            success: true,
            data: None,
            error: None,
        }
    }
}

/// Helper functions for creating error responses
pub fn api_error(code: i32, message: impl Into<String>) -> ApiResponse<()> {
    ApiResponse {
        success: false,
        data: None,
        error: Some(ErrorInfo {
            code,
            message: message.into(),
            details: None,
        }),
    }
}

/// Helper function for creating error responses with details
pub fn api_error_with_details(
    code: i32,
    message: impl Into<String>,
    details: serde_json::Value,
) -> ApiResponse<()> {
    ApiResponse {
        success: false,
        data: None,
        error: Some(ErrorInfo {
            code,
            message: message.into(),
            details: Some(details),
        }),
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = if self.success {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };
        (status, Json(self)).into_response()
    }
}

/// Response wrapper for list data
#[derive(Debug, Serialize)]
pub struct ListData<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
}

impl<T: Serialize> ListData<T> {
    pub fn new(items: Vec<T>, total: i64) -> Self {
        Self { items, total }
    }

    /// Create from a vector, using its length as total
    pub fn from_vec(items: Vec<T>) -> Self {
        let total = items.len() as i64;
        Self { items, total }
    }
}

/// Response wrapper for paginated data
#[derive(Debug, Serialize)]
pub struct PaginatedData<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

impl<T: Serialize> PaginatedData<T> {
    pub fn new(items: Vec<T>, total: i64, page: i32, page_size: i32) -> Self {
        let total_pages = if page_size > 0 {
            ((total as f64) / (page_size as f64)).ceil() as i32
        } else {
            0
        };

        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

/// Helper trait for converting data into ApiResponse
pub trait IntoApiResponse<T: Serialize> {
    fn into_api_response(self) -> ApiResponse<T>;
}

impl<T: Serialize> IntoApiResponse<T> for T {
    fn into_api_response(self) -> ApiResponse<T> {
        ApiResponse::success(self)
    }
}

impl<T: Serialize> IntoApiResponse<ListData<T>> for Vec<T> {
    fn into_api_response(self) -> ApiResponse<ListData<T>> {
        ApiResponse::success(ListData::from_vec(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, PartialEq)]
    struct TestData {
        id: i32,
        name: String,
    }

    #[test]
    fn test_success_response() {
        let data = TestData {
            id: 1,
            name: "test".to_string(),
        };
        let response = ApiResponse::success(data);

        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());

        let data = response.data.unwrap();
        assert_eq!(data.id, 1);
        assert_eq!(data.name, "test");
    }

    #[test]
    fn test_ok_response() {
        let response: ApiResponse<()> = ApiResponse::<()>::ok();

        assert!(response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_error_response() {
        let response = api_error(1001, "Validation failed");

        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, 1001);
        assert_eq!(error.message, "Validation failed");
        assert!(error.details.is_none());
    }

    #[test]
    fn test_error_with_details() {
        let details = serde_json::json!({
            "field": "email",
            "reason": "invalid format"
        });
        let response = api_error_with_details(1001, "Validation failed", details.clone());

        assert!(!response.success);
        let error = response.error.unwrap();
        assert_eq!(error.details, Some(details));
    }

    #[test]
    fn test_list_data() {
        let items = vec![
            TestData {
                id: 1,
                name: "a".to_string(),
            },
            TestData {
                id: 2,
                name: "b".to_string(),
            },
        ];
        let list = ListData::new(items, 100);

        assert_eq!(list.items.len(), 2);
        assert_eq!(list.total, 100);
    }

    #[test]
    fn test_list_data_from_vec() {
        let items = vec![
            TestData {
                id: 1,
                name: "a".to_string(),
            },
            TestData {
                id: 2,
                name: "b".to_string(),
            },
        ];
        let list = ListData::from_vec(items);

        assert_eq!(list.items.len(), 2);
        assert_eq!(list.total, 2);
    }

    #[test]
    fn test_paginated_data() {
        let items = vec![TestData {
            id: 1,
            name: "a".to_string(),
        }];
        let paginated = PaginatedData::new(items, 100, 1, 10);

        assert_eq!(paginated.items.len(), 1);
        assert_eq!(paginated.total, 100);
        assert_eq!(paginated.page, 1);
        assert_eq!(paginated.page_size, 10);
        assert_eq!(paginated.total_pages, 10);
    }

    #[test]
    fn test_paginated_data_total_pages_calculation() {
        // 95 items with page_size 10 = 10 pages
        let paginated = PaginatedData::new(vec![1, 2, 3], 95, 1, 10);
        assert_eq!(paginated.total_pages, 10);

        // 100 items with page_size 10 = 10 pages
        let paginated = PaginatedData::new(vec![1, 2, 3], 100, 1, 10);
        assert_eq!(paginated.total_pages, 10);

        // 101 items with page_size 10 = 11 pages
        let paginated = PaginatedData::new(vec![1, 2, 3], 101, 1, 10);
        assert_eq!(paginated.total_pages, 11);
    }

    #[test]
    fn test_serialization() {
        let response = ApiResponse::success(TestData {
            id: 1,
            name: "test".to_string(),
        });
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"name\":\"test\""));
        assert!(!json.contains("\"error\"")); // error should be skipped when None
    }
}

//! API response wrapper for consistent JSON responses.
//!
//! Provides [`ApiResponse`] with success status, message, and optional data payload.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// Standard API response envelope with HTTP status code and optional data.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    /// HTTP status code (e.g., 200, 201).
    pub success: u16,
    /// Human-readable status message.
    pub message: String,
    /// Optional response payload. Omitted when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Creates a success response with HTTP 200 and the given data.
    ///
    /// # Examples
    ///
    /// ```
    /// use capeos_common::models::ApiResponse;
    ///
    /// let resp = ApiResponse::ok("hello");
    /// assert_eq!(resp.success, 200);
    /// assert_eq!(resp.data, Some("hello"));
    /// ```
    pub fn ok(data: T) -> Self {
        Self {
            success: 200,
            message: "ok".to_string(),
            data: Some(data),
        }
    }

    /// Creates a created response with HTTP 201 and the given data.
    pub fn created(data: T) -> Self {
        Self {
            success: 201,
            message: "created".to_string(),
            data: Some(data),
        }
    }
}

impl ApiResponse<()> {
    /// Creates a successful response with HTTP 200 and no data payload.
    pub fn ok_empty() -> Self {
        Self {
            success: 200,
            message: "ok".to_string(),
            data: None,
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.success).unwrap_or(StatusCode::OK);
        (status, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_ok() {
        let resp = ApiResponse::ok("hello");
        assert_eq!(resp.success, 200);
        assert_eq!(resp.message, "ok");
        assert_eq!(resp.data, Some("hello"));
    }

    #[test]
    fn test_api_response_created() {
        let resp = ApiResponse::created(42);
        assert_eq!(resp.success, 201);
        assert_eq!(resp.message, "created");
        assert_eq!(resp.data, Some(42));
    }

    #[test]
    fn test_api_response_ok_empty() {
        let resp = ApiResponse::ok_empty();
        assert_eq!(resp.success, 200);
        assert_eq!(resp.message, "ok");
        assert!(resp.data.is_none());
    }

    #[test]
    fn test_api_response_serialization() {
        let resp = ApiResponse::ok(vec![1, 2, 3]);
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["success"], 200);
        assert_eq!(parsed["message"], "ok");
        assert_eq!(parsed["data"], serde_json::json!([1, 2, 3]));
    }
}

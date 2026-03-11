//! Application error types and HTTP response handling.
//!
//! Provides [`AppError`] for consistent error handling across CapeOS services,
//! with automatic conversion to JSON HTTP responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// Application-level errors that map to HTTP status codes.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Returned when a requested resource does not exist (HTTP 404).
    #[error("not found: {0}")]
    NotFound(String),
    /// Returned when the request lacks valid authentication (HTTP 401).
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    /// Returned when the request is malformed or invalid (HTTP 400).
    #[error("bad request: {0}")]
    BadRequest(String),
    /// Returned for unexpected server-side failures (HTTP 500).
    #[error("internal: {0}")]
    Internal(String),
    /// Wraps any [`anyhow::Error`] as an internal server error (HTTP 500).
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Anyhow(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        let body = Json(json!({
            "success": status.as_u16(),
            "message": message,
            "data": null
        }));
        (status, body).into_response()
    }
}

/// Result type alias for operations that can fail with [`AppError`].
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    async fn status_and_body_from_response(response: Response) -> (StatusCode, serde_json::Value) {
        let status = response.status();
        let (_, body) = response.into_parts();
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        (status, json)
    }

    #[tokio::test]
    async fn test_not_found_response() {
        let err = AppError::NotFound("resource missing".to_string());
        let response = err.into_response();
        let (status, json) = status_and_body_from_response(response).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["message"], "resource missing");
    }

    #[tokio::test]
    async fn test_unauthorized_response() {
        let err = AppError::Unauthorized("invalid token".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_bad_request_response() {
        let err = AppError::BadRequest("invalid input".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_internal_response() {
        let err = AppError::Internal("server error".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_anyhow_response() {
        let err = AppError::Anyhow(anyhow::anyhow!("test"));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}

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

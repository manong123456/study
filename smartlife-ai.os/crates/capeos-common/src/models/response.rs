use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: 200,
            message: "ok".to_string(),
            data: Some(data),
        }
    }

    pub fn created(data: T) -> Self {
        Self {
            success: 201,
            message: "created".to_string(),
            data: Some(data),
        }
    }
}

impl ApiResponse<()> {
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

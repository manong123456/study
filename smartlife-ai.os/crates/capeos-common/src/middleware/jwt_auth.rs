//! JWT authentication middleware for Axum.
//!
//! Extracts and validates JWT tokens from the `Authorization` header or `token` query parameter.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};

/// JWT claims payload decoded from the token.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (typically user identifier).
    pub sub: String,
    /// User ID.
    pub id: i64,
    /// Expiration timestamp (Unix seconds).
    pub exp: usize,
    /// Issued-at timestamp (Unix seconds).
    pub iat: usize,
}

/// JWT authentication middleware.
///
/// Extracts the token from:
/// - `Authorization: Bearer <token>` header
/// - `token` query parameter
///
/// Skips authentication for localhost requests. Returns `401 Unauthorized` if no valid token
/// is present. Token validation against a JWKS endpoint is planned (TODO).
pub async fn jwt_auth(req: Request, next: Next) -> Result<Response, StatusCode> {
    let token = extract_token(&req);
    if token.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    // TODO: validate token against UserService JWKS endpoint
    Ok(next.run(req).await)
}

/// Extracts the JWT token from the request.
///
/// Checks the `Authorization: Bearer <token>` header first, then the `token` query parameter.
fn extract_token(req: &Request) -> Option<String> {
    if let Some(auth) = req.headers().get("Authorization") {
        if let Ok(val) = auth.to_str() {
            if let Some(token) = val.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }
    if let Some(query) = req.uri().query() {
        for part in query.split('&') {
            if let Some(value) = part.strip_prefix("token=") {
                return Some(value.to_string());
            }
        }
    }
    None
}

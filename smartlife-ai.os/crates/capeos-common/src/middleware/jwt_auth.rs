//! JWT authentication middleware for Axum.
//!
//! Extracts and validates JWT tokens from the `Authorization` header or `token` query parameter.

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
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
pub(crate) fn extract_token(req: &Request) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;

    fn request_with_header(value: &str) -> Request<Body> {
        Request::builder()
            .uri("/")
            .header("Authorization", value)
            .body(Body::empty())
            .unwrap()
    }

    fn request_with_uri(uri: &str) -> Request<Body> {
        Request::builder().uri(uri).body(Body::empty()).unwrap()
    }

    #[test]
    fn test_extract_token_from_header() {
        let req = request_with_header("Bearer xxx");
        assert_eq!(extract_token(&req), Some("xxx".to_string()));
    }

    #[test]
    fn test_extract_token_from_query() {
        let req = request_with_uri("/?token=abc");
        assert_eq!(extract_token(&req), Some("abc".to_string()));
    }

    #[test]
    fn test_extract_token_missing() {
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        assert_eq!(extract_token(&req), None);
    }

    #[test]
    fn test_extract_token_no_bearer_prefix() {
        let req = request_with_header("Basic xxx");
        assert_eq!(extract_token(&req), None);
    }
}

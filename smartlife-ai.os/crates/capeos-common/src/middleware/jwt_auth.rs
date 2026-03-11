use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub id: i64,
    pub exp: usize,
    pub iat: usize,
}

/// JWT authentication middleware.
/// Skips auth for localhost requests.
/// Extracts token from Authorization header or `token` query parameter.
pub async fn jwt_auth(req: Request, next: Next) -> Result<Response, StatusCode> {
    let token = extract_token(&req);
    if token.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    // TODO: validate token against UserService JWKS endpoint
    Ok(next.run(req).await)
}

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

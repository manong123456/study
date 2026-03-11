//! HTTP middleware for CapeOS services.
//!
//! Provides CORS, compression, and JWT authentication layers for Axum applications.

pub mod jwt_auth;

use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};

/// Returns a CORS layer that allows any origin, method, and header with a 48-hour max-age.
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .max_age(std::time::Duration::from_secs(172800))
}

/// Returns a gzip/brotli compression layer for response bodies.
pub fn compression_layer() -> CompressionLayer {
    CompressionLayer::new()
}

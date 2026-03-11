pub mod jwt_auth;

use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .max_age(std::time::Duration::from_secs(172800))
}

pub fn compression_layer() -> CompressionLayer {
    CompressionLayer::new()
}

//! CapeOS User Service
//!
//! Provides user authentication, registration, and JWT token issuance for the CapeOS platform.
//! Exposes JWKS (JSON Web Key Set) at `/.well-known/jwks.json` for public key distribution
//! to enable JWT verification by other services.

mod handlers;
mod db;
mod jwt_issuer;

use std::net::SocketAddr;
use std::sync::Arc;
use axum::Router;
use axum::routing::{get, post, put, delete};
use capeos_common::paths::DEFAULT_RUNTIME_PATH;
use capeos_common::utils::{write_url_file, port::get_available_port};
use capeos_common::utils::service_discovery::register_routes;
use tracing_subscriber::EnvFilter;

/// Shared application state passed to request handlers.
pub struct AppState {
    /// SQLite database connection for user storage.
    pub db: tokio_rusqlite::Connection,
    /// JWT issuer for token creation and JWKS distribution.
    pub jwt: jwt_issuer::JwtIssuer,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let conn = db::init_db("/var/lib/capeos/db").await?;
    let jwt = jwt_issuer::JwtIssuer::new()?;
    let state = Arc::new(AppState { db: conn, jwt });
    let port = get_available_port();

    let app = Router::new()
        .route("/ping", get(|| async { "pong" }))
        .route("/v1/user_service/users/register", post(handlers::register))
        .route("/v1/user_service/users/login", post(handlers::login))
        .route("/v1/user_service/users/current", get(handlers::current_user))
        .route("/v1/user_service/users/current/password", put(handlers::change_password))
        .route("/v1/user_service/users/status", get(handlers::user_status))
        .route("/v1/user_service/users/current", delete(handlers::delete_user))
        .route("/.well-known/jwks.json", get(handlers::jwks))
        .with_state(state);

    let listen_url = format!("http://127.0.0.1:{}", port);
    write_url_file(DEFAULT_RUNTIME_PATH, "user-service.url", &listen_url).await?;

    tokio::spawn({
        let listen_url = listen_url.clone();
        async move {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            if let Ok(mgmt) = capeos_common::utils::service_discovery::get_service_address(DEFAULT_RUNTIME_PATH, "management.url").await {
                let routes = vec![
                    capeos_common::models::Route { path: "/v1/user_service".to_string(), target: listen_url.clone() },
                    capeos_common::models::Route { path: "/.well-known".to_string(), target: listen_url },
                ];
                let _ = register_routes(&mgmt, &routes).await;
            }
        }
    });

    tracing::info!("UserService listening on 127.0.0.1:{}", port);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

//! # CapeOS Message Bus Binary
//!
//! Event pub/sub service for the CapeOS runtime. Provides:
//! - Event type registration and listing
//! - Event publishing (HTTP POST)
//! - WebSocket subscriptions for real-time event delivery
//! - Service discovery via `message-bus.url` and gateway route registration

mod handlers;
mod store;

use axum::routing::{get, post};
use axum::Router;
use capeos_common::paths::DEFAULT_RUNTIME_PATH;
use capeos_common::utils::service_discovery::register_routes;
use capeos_common::utils::{port::get_available_port, write_url_file};
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let state = store::BusState::new();
    let port = get_available_port();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let app = Router::new()
        .route("/ping", get(|| async { "pong" }))
        .route(
            "/v1/message_bus/event_types",
            get(handlers::list_event_types).post(handlers::register_event_types),
        )
        .route(
            "/v1/message_bus/event/:source_id/:name",
            post(handlers::publish_event),
        )
        .route("/v1/message_bus/subscribe", get(handlers::subscribe))
        .with_state(state);

    let listen_url = format!("http://127.0.0.1:{}", port);
    write_url_file(DEFAULT_RUNTIME_PATH, "message-bus.url", &listen_url).await?;

    // Register with gateway
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        if let Ok(mgmt_url) = capeos_common::utils::service_discovery::get_service_address(
            DEFAULT_RUNTIME_PATH,
            "management.url",
        )
        .await
        {
            let routes = vec![capeos_common::models::Route {
                path: "/v1/message_bus".to_string(),
                target: format!("http://127.0.0.1:{}", port),
            }];
            let _ = register_routes(&mgmt_url, &routes).await;
        }
    });

    tracing::info!("MessageBus listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

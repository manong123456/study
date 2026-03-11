//! # CapeOS Gateway Binary
//!
//! The gateway serves as a reverse proxy and management API for the CapeOS runtime.
//! On startup it:
//! - Binds a management API on localhost (internal) for route registration and health checks
//! - Binds the gateway on all interfaces (external) to proxy incoming HTTP requests to backend services
//! - Writes URL files for service discovery (`management.url`, `gateway.url`)
//! - Runs both servers concurrently via `tokio::select!`

mod management;
mod proxy;
mod route_table;

use capeos_common::paths::DEFAULT_RUNTIME_PATH;
use capeos_common::utils::{port::get_available_port, write_url_file};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let table = route_table::RouteTable::new();
    let mgmt_port = get_available_port();

    // Management API (internal only)
    let mgmt_app = management::management_router(table.clone());
    let mgmt_addr = SocketAddr::from(([127, 0, 0, 1], mgmt_port));

    // Gateway (external)
    let gateway_port: u16 = std::env::var("GATEWAY_PORT")
        .unwrap_or_else(|_| "80".to_string())
        .parse()
        .unwrap_or(80);
    let gateway_app = proxy::proxy_router(table.clone()).layer(TraceLayer::new_for_http());
    let gateway_addr = SocketAddr::from(([0, 0, 0, 0], gateway_port));

    // Write URL files for service discovery
    let mgmt_url = format!("http://127.0.0.1:{}", mgmt_port);
    let gateway_url = format!("http://0.0.0.0:{}", gateway_port);
    write_url_file(DEFAULT_RUNTIME_PATH, "management.url", &mgmt_url).await?;
    write_url_file(DEFAULT_RUNTIME_PATH, "gateway.url", &gateway_url).await?;

    tracing::info!("Gateway listening on {}", gateway_addr);
    tracing::info!("Management API on {}", mgmt_addr);

    let mgmt_listener = tokio::net::TcpListener::bind(mgmt_addr).await?;
    let gw_listener = tokio::net::TcpListener::bind(gateway_addr).await?;

    tokio::select! {
        r = axum::serve(mgmt_listener, mgmt_app) => r?,
        r = axum::serve(gw_listener, gateway_app) => r?,
    }

    Ok(())
}

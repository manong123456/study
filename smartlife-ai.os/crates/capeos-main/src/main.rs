//! CapeOS Main Service
//!
//! Core CapeOS service providing system monitoring, file and folder management,
//! health checks, and routing for other CapeOS components.

mod server;

use std::net::SocketAddr;
use axum::Router;
use axum::routing::get;
use capeos_common::paths::DEFAULT_RUNTIME_PATH;
use capeos_common::utils::{write_url_file, port::get_available_port};
use capeos_common::utils::service_discovery::register_routes;
use capeos_common::middleware;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let state = server::state::AppState::new().await?;
    let port = get_available_port();

    let app = Router::new()
        .route("/ping", get(|| async { "pong" }))
        .nest("/v1/sys", server::api::v1::sys_routes(state.clone()))
        .nest("/v1/file", server::api::v1::file_routes(state.clone()))
        .nest("/v1/folder", server::api::v1::folder_routes(state.clone()))
        .nest("/v1/capeos", server::api::v1::capeos_routes(state.clone()))
        .layer(middleware::cors_layer())
        .layer(middleware::compression_layer());

    let listen_url = format!("http://127.0.0.1:{}", port);
    write_url_file(DEFAULT_RUNTIME_PATH, "capeos.url", &listen_url).await?;

    tokio::spawn({
        let listen_url = listen_url.clone();
        async move {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            if let Ok(mgmt) = capeos_common::utils::service_discovery::get_service_address(DEFAULT_RUNTIME_PATH, "management.url").await {
                let routes = vec![
                    capeos_common::models::Route { path: "/v1/sys".to_string(), target: listen_url.clone() },
                    capeos_common::models::Route { path: "/v1/file".to_string(), target: listen_url.clone() },
                    capeos_common::models::Route { path: "/v1/folder".to_string(), target: listen_url.clone() },
                    capeos_common::models::Route { path: "/v1/capeos".to_string(), target: listen_url.clone() },
                    capeos_common::models::Route { path: "/v1/port".to_string(), target: listen_url.clone() },
                    capeos_common::models::Route { path: "/v1/image".to_string(), target: listen_url.clone() },
                    capeos_common::models::Route { path: "/v1/batch".to_string(), target: listen_url.clone() },
                    capeos_common::models::Route { path: "/v1/samba".to_string(), target: listen_url.clone() },
                    capeos_common::models::Route { path: "/v1/notify".to_string(), target: listen_url.clone() },
                    capeos_common::models::Route { path: "/v1/cloud".to_string(), target: listen_url.clone() },
                    capeos_common::models::Route { path: "/v1/driver".to_string(), target: listen_url },
                ];
                let _ = register_routes(&mgmt, &routes).await;
            }
        }
    });

    tracing::info!("CapeOS main service on 127.0.0.1:{}", port);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

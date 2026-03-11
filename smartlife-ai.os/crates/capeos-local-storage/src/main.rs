//! CapeOS Local Storage Service
//!
//! Provides disk management, USB detection, and MergerFS integration for the CapeOS platform.
//! Exposes HTTP endpoints for querying disk information and storage status.

use axum::{routing::get, Json, Router};
use capeos_common::models::ApiResponse;
use capeos_common::paths::DEFAULT_RUNTIME_PATH;
use capeos_common::utils::{port::get_available_port, write_url_file};
use serde::Serialize;
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

/// Information about a disk or storage device.
#[derive(Serialize)]
struct DiskInfo {
    /// Device or disk name (e.g., `/dev/sda1`).
    name: String,
    /// Mount point path (e.g., `/mnt/data`).
    mount_point: String,
    /// Total space in bytes.
    total_space: u64,
    /// Available (free) space in bytes.
    available_space: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let port = get_available_port();
    let app = Router::new()
        .route("/ping", get(|| async { "pong" }))
        .route("/v1/local_storage/disks", get(list_disks));

    let listen_url = format!("http://127.0.0.1:{}", port);
    write_url_file(DEFAULT_RUNTIME_PATH, "local-storage.url", &listen_url).await?;

    tracing::info!("LocalStorage on 127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Lists all disks and storage devices detected by the system.
async fn list_disks() -> Json<ApiResponse<Vec<DiskInfo>>> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let list: Vec<DiskInfo> = disks
        .list()
        .iter()
        .map(|d| DiskInfo {
            name: d.name().to_string_lossy().to_string(),
            mount_point: d.mount_point().to_string_lossy().to_string(),
            total_space: d.total_space(),
            available_space: d.available_space(),
        })
        .collect();
    Json(ApiResponse::ok(list))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_list_disks_handler() {
        let app = Router::new().route("/v1/local_storage/disks", get(list_disks));
        let req = Request::builder()
            .uri("/v1/local_storage/disks")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}

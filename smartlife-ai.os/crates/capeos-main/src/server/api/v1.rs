//! API v1 route definitions and handlers.
//!
//! System, file, folder, and CapeOS health endpoints.

use crate::server::services::system;
use crate::server::state::AppState;
use axum::{extract::Query, extract::State, routing::get, Json, Router};
use capeos_common::models::ApiResponse;
use serde::Deserialize;

/// Builds the `/v1/sys` router: version, hardware, utilization.
pub fn sys_routes(state: AppState) -> Router {
    Router::new()
        .route("/version/current", get(get_version))
        .route("/hardware", get(get_hardware))
        .route("/utilization", get(get_utilization))
        .with_state(state)
}

/// Builds the `/v1/file` router: file content reading.
pub fn file_routes(state: AppState) -> Router {
    Router::new()
        .route("/content", get(get_file_content))
        .with_state(state)
}

/// Builds the `/v1/folder` router: directory listing.
pub fn folder_routes(state: AppState) -> Router {
    Router::new().route("/", get(list_dir)).with_state(state)
}

/// Builds the `/v1/capeos` router: health and service status.
pub fn capeos_routes(state: AppState) -> Router {
    Router::new()
        .route("/health/services", get(health_services))
        .with_state(state)
}

/// GET `/v1/sys/version/current` - Returns the current CapeOS version.
async fn get_version() -> Json<ApiResponse<String>> {
    Json(ApiResponse::ok(capeos_common::VERSION.to_string()))
}

/// GET `/v1/sys/hardware` - Returns hardware and OS information.
async fn get_hardware() -> Json<ApiResponse<system::HardwareInfo>> {
    Json(ApiResponse::ok(system::get_hardware_info()))
}

/// GET `/v1/sys/utilization` - Returns CPU and memory utilization.
async fn get_utilization(
    State(state): State<AppState>,
) -> Json<ApiResponse<system::SystemUtilization>> {
    Json(ApiResponse::ok(system::get_utilization(&state.sys)))
}

/// Query parameter for path-based endpoints (file content, directory listing).
#[derive(Deserialize)]
pub struct PathQuery {
    /// Optional path; defaults to `/` if not provided.
    pub path: Option<String>,
}

/// GET `/v1/file/content` - Returns the contents of a file at the given path.
///
/// Query: `path` (optional, default `/`). Returns raw file content as string.
async fn get_file_content(
    Query(q): Query<PathQuery>,
) -> Result<String, capeos_common::error::AppError> {
    let path = q.path.unwrap_or_else(|| "/".to_string());
    tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| capeos_common::error::AppError::NotFound(e.to_string()))
}

/// A single entry in a directory listing.
#[derive(serde::Serialize)]
pub struct DirEntry {
    /// File or directory name.
    pub name: String,
    /// True if this entry is a directory.
    pub is_dir: bool,
    /// File size in bytes (0 for directories).
    pub size: u64,
}

/// GET `/v1/folder/` - Lists directory contents at the given path.
///
/// Query: `path` (optional, default `/`). Returns `Vec<DirEntry>`.
async fn list_dir(
    Query(q): Query<PathQuery>,
) -> Result<Json<ApiResponse<Vec<DirEntry>>>, capeos_common::error::AppError> {
    let path = q.path.unwrap_or_else(|| "/".to_string());
    let mut entries = Vec::new();
    let mut dir = tokio::fs::read_dir(&path)
        .await
        .map_err(|e| capeos_common::error::AppError::NotFound(e.to_string()))?;
    while let Ok(Some(entry)) = dir.next_entry().await {
        let meta = entry.metadata().await.ok();
        entries.push(DirEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            is_dir: meta.as_ref().map(|m| m.is_dir()).unwrap_or(false),
            size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
        });
    }
    Ok(Json(ApiResponse::ok(entries)))
}

/// GET `/v1/capeos/health/services` - Returns status of CapeOS services.
///
/// Checks systemctl for capeos-* services. Response: `HealthServices` with running/not_running lists.
async fn health_services() -> Json<ApiResponse<system::HealthServices>> {
    // Simple check - list capeos-* services
    let services = vec![
        "capeos-gateway",
        "capeos-message-bus",
        "capeos-user-service",
        "capeos-local-storage",
        "capeos-app-management",
        "capeos",
    ];
    let mut running = Vec::new();
    let mut not_running = Vec::new();
    for svc in services {
        let name = format!("{}.service", svc);
        let output = tokio::process::Command::new("systemctl")
            .args(["is-active", &name])
            .output()
            .await;
        match output {
            Ok(o) if String::from_utf8_lossy(&o.stdout).trim() == "active" => running.push(name),
            _ => not_running.push(name),
        }
    }
    Json(ApiResponse::ok(system::HealthServices {
        running,
        not_running,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    async fn test_router() -> axum::Router {
        let state = crate::server::state::AppState::new().await.unwrap();
        axum::Router::new()
            .nest("/v1/sys", sys_routes(state.clone()))
            .nest("/v1/folder", folder_routes(state.clone()))
            .nest("/v1/capeos", capeos_routes(state.clone()))
    }

    #[tokio::test]
    async fn test_get_version() {
        let app = test_router().await;
        let req = Request::builder()
            .uri("/v1/sys/version/current")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("version") || body_str.contains("data"));
    }

    #[tokio::test]
    async fn test_get_hardware() {
        let app = test_router().await;
        let req = Request::builder()
            .uri("/v1/sys/hardware")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("cpu_count"));
    }

    #[tokio::test]
    async fn test_get_utilization() {
        let app = test_router().await;
        let req = Request::builder()
            .uri("/v1/sys/utilization")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_dir() {
        let app = test_router().await;
        let req = Request::builder()
            .uri("/v1/folder?path=/tmp")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("data") || body_str.contains("["));
    }

    #[tokio::test]
    async fn test_list_dir_nonexistent() {
        let app = test_router().await;
        let req = Request::builder()
            .uri("/v1/folder?path=/nonexistent_dir_xyz")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_health_services() {
        let app = test_router().await;
        let req = Request::builder()
            .uri("/v1/capeos/health/services")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}

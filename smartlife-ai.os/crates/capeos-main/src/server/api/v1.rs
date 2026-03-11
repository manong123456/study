//! API v1 route definitions and handlers.
//!
//! System, file, folder, and CapeOS health endpoints.

use axum::{Router, routing::get, Json, extract::State, extract::Query};
use serde::Deserialize;
use capeos_common::models::ApiResponse;
use crate::server::state::AppState;
use crate::server::services::system;

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
    Router::new()
        .route("/", get(list_dir))
        .with_state(state)
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
async fn get_utilization(State(state): State<AppState>) -> Json<ApiResponse<system::SystemUtilization>> {
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
async fn get_file_content(Query(q): Query<PathQuery>) -> Result<String, capeos_common::error::AppError> {
    let path = q.path.unwrap_or_else(|| "/".to_string());
    tokio::fs::read_to_string(&path).await
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
async fn list_dir(Query(q): Query<PathQuery>) -> Result<Json<ApiResponse<Vec<DirEntry>>>, capeos_common::error::AppError> {
    let path = q.path.unwrap_or_else(|| "/".to_string());
    let mut entries = Vec::new();
    let mut dir = tokio::fs::read_dir(&path).await
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
    Json(ApiResponse::ok(system::HealthServices { running, not_running }))
}

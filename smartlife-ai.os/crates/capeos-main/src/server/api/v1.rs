use axum::{Router, routing::get, Json, extract::State, extract::Query};
use serde::Deserialize;
use capeos_common::models::ApiResponse;
use crate::server::state::AppState;
use crate::server::services::system;

pub fn sys_routes(state: AppState) -> Router {
    Router::new()
        .route("/version/current", get(get_version))
        .route("/hardware", get(get_hardware))
        .route("/utilization", get(get_utilization))
        .with_state(state)
}

pub fn file_routes(state: AppState) -> Router {
    Router::new()
        .route("/content", get(get_file_content))
        .with_state(state)
}

pub fn folder_routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_dir))
        .with_state(state)
}

pub fn capeos_routes(state: AppState) -> Router {
    Router::new()
        .route("/health/services", get(health_services))
        .with_state(state)
}

async fn get_version() -> Json<ApiResponse<String>> {
    Json(ApiResponse::ok(capeos_common::VERSION.to_string()))
}

async fn get_hardware() -> Json<ApiResponse<system::HardwareInfo>> {
    Json(ApiResponse::ok(system::get_hardware_info()))
}

async fn get_utilization(State(state): State<AppState>) -> Json<ApiResponse<system::SystemUtilization>> {
    Json(ApiResponse::ok(system::get_utilization(&state.sys)))
}

#[derive(Deserialize)]
pub struct PathQuery {
    pub path: Option<String>,
}

async fn get_file_content(Query(q): Query<PathQuery>) -> Result<String, capeos_common::error::AppError> {
    let path = q.path.unwrap_or_else(|| "/".to_string());
    tokio::fs::read_to_string(&path).await
        .map_err(|e| capeos_common::error::AppError::NotFound(e.to_string()))
}

#[derive(serde::Serialize)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

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

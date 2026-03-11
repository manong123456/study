//! Service discovery utilities.
//!
//! Reads service URLs from runtime files and registers routes with the gateway.

use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, Result};
use tracing::info;

use crate::models::Route;

/// Reads a service URL from a file in the runtime directory.
///
/// # Arguments
///
/// * `runtime_path` - Base directory for runtime files (e.g., `/var/run/capeos`).
/// * `filename` - Name of the file containing the URL (e.g., `gateway.url`).
///
/// # Returns
///
/// The trimmed URL string from the file.
pub async fn get_service_address(runtime_path: &str, filename: &str) -> Result<String> {
    let path = Path::new(runtime_path).join(filename);
    let content = tokio::fs::read_to_string(&path).await?;
    Ok(content.trim().to_string())
}

/// Polls until `get_service_address` succeeds and the service responds to `/ping`.
///
/// # Arguments
///
/// * `runtime_path` - Base directory for runtime files.
/// * `filename` - Name of the file containing the service URL.
/// * `retries` - Maximum number of attempts before giving up.
///
/// # Returns
///
/// The service URL when it becomes available, or an error after all retries.
pub async fn wait_for_service(runtime_path: &str, filename: &str, retries: u32) -> Result<String> {
    for i in 0..retries {
        match get_service_address(runtime_path, filename).await {
            Ok(addr) => {
                let ping_url = format!("{}/ping", addr.trim_end_matches('/'));
                if reqwest::get(&ping_url).await.is_ok() {
                    return Ok(addr);
                }
            }
            Err(_) => {}
        }
        info!("Waiting for {} ({}/{})", filename, i + 1, retries);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Err(anyhow!("{} not available after {} retries", filename, retries))
}

/// Registers routes with the gateway management API.
///
/// POSTs each route to `{management_url}/v1/gateway/routes`.
///
/// # Arguments
///
/// * `management_url` - Base URL of the gateway management API (e.g., `http://127.0.0.1:8080`).
/// * `routes` - Slice of routes to register.
pub async fn register_routes(management_url: &str, routes: &[Route]) -> Result<()> {
    let client = reqwest::Client::new();
    for route in routes {
        let url = format!("{}/v1/gateway/routes", management_url.trim_end_matches('/'));
        client.post(&url).json(route).send().await?;
    }
    Ok(())
}

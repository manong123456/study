use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, Result};
use tracing::info;

use crate::models::Route;

pub async fn get_service_address(runtime_path: &str, filename: &str) -> Result<String> {
    let path = Path::new(runtime_path).join(filename);
    let content = tokio::fs::read_to_string(&path).await?;
    Ok(content.trim().to_string())
}

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

pub async fn register_routes(management_url: &str, routes: &[Route]) -> Result<()> {
    let client = reqwest::Client::new();
    for route in routes {
        let url = format!("{}/v1/gateway/routes", management_url.trim_end_matches('/'));
        client.post(&url).json(route).send().await?;
    }
    Ok(())
}

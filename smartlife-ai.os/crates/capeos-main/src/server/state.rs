//! Application state shared across request handlers.

use std::sync::Arc;
use anyhow::Result;

/// Shared application state for the CapeOS main service.
#[derive(Clone)]
pub struct AppState {
    /// System information (CPU, memory, etc.) for utilization and hardware endpoints.
    pub sys: Arc<sysinfo::System>,
}

impl AppState {
    /// Creates new application state with refreshed system info.
    pub async fn new() -> Result<Self> {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        Ok(Self { sys: Arc::new(sys) })
    }
}

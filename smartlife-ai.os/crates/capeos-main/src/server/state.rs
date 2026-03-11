use std::sync::Arc;
use anyhow::Result;

#[derive(Clone)]
pub struct AppState {
    pub sys: Arc<sysinfo::System>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        Ok(Self { sys: Arc::new(sys) })
    }
}

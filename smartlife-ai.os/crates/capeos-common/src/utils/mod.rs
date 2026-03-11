pub mod port;
pub mod service_discovery;

use std::path::Path;

pub async fn ensure_dir(path: &str) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(path).await?;
    Ok(())
}

pub async fn write_url_file(runtime_path: &str, filename: &str, url: &str) -> anyhow::Result<()> {
    ensure_dir(runtime_path).await?;
    let path = Path::new(runtime_path).join(filename);
    tokio::fs::write(&path, url).await?;
    Ok(())
}

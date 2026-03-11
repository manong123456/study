//! Utility functions for CapeOS services.
//!
//! Includes port allocation, service discovery, and filesystem helpers.

pub mod port;
pub mod service_discovery;

use std::path::Path;

/// Creates the directory and all parent directories if they do not exist.
///
/// # Arguments
///
/// * `path` - The directory path to create (e.g., `/var/lib/capeos`).
pub async fn ensure_dir(path: &str) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(path).await?;
    Ok(())
}

/// Writes a URL to a file in the runtime directory.
///
/// Creates the runtime directory if needed, then writes the URL string to
/// `{runtime_path}/{filename}`.
///
/// # Arguments
///
/// * `runtime_path` - Base directory for runtime files (e.g., `/var/run/capeos`).
/// * `filename` - Name of the file to write (e.g., `gateway.url`).
/// * `url` - The URL string to write (e.g., `http://127.0.0.1:8080`).
pub async fn write_url_file(runtime_path: &str, filename: &str, url: &str) -> anyhow::Result<()> {
    ensure_dir(runtime_path).await?;
    let path = Path::new(runtime_path).join(filename);
    tokio::fs::write(&path, url).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ensure_dir() {
        let path = "/tmp/capeos_test_ensure_dir_54321";
        ensure_dir(path).await.unwrap();
        assert!(std::path::Path::new(path).is_dir());
        tokio::fs::remove_dir(path).await.ok();
    }

    #[tokio::test]
    async fn test_write_url_file() {
        let runtime_path = "/tmp/capeos_test_write_url_67890";
        let filename = "test.url";
        let url = "http://127.0.0.1:8080";
        write_url_file(runtime_path, filename, url).await.unwrap();
        let path = Path::new(runtime_path).join(filename);
        let content = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(content, url);
        tokio::fs::remove_file(&path).await.ok();
        tokio::fs::remove_dir(runtime_path).await.ok();
    }
}

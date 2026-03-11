pub mod error;
pub mod middleware;
pub mod models;
pub mod utils;

pub mod paths {
    pub const DEFAULT_CONFIG_PATH: &str = "/etc/capeos";
    pub const DEFAULT_DATA_PATH: &str = "/var/lib/capeos";
    pub const DEFAULT_FILE_PATH: &str = "/var/lib/capeos/files";
    pub const DEFAULT_LOG_PATH: &str = "/var/log/capeos";
    pub const DEFAULT_RUNTIME_PATH: &str = "/var/run/capeos";
    pub const DEFAULT_DB_PATH: &str = "/var/lib/capeos/db";
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

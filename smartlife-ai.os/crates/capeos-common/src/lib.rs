//! # CapeOS Common
//!
//! Shared types, utilities, and middleware for CapeOS services.
//! This crate provides error handling, API response models, JWT authentication,
//! path constants, and service discovery helpers.

pub mod error;
pub mod middleware;
pub mod models;
pub mod utils;

/// Default filesystem paths used by CapeOS services.
pub mod paths {
    /// Default configuration directory path.
    pub const DEFAULT_CONFIG_PATH: &str = "/etc/capeos";
    /// Default data directory path.
    pub const DEFAULT_DATA_PATH: &str = "/var/lib/capeos";
    /// Default file storage directory path.
    pub const DEFAULT_FILE_PATH: &str = "/var/lib/capeos/files";
    /// Default log directory path.
    pub const DEFAULT_LOG_PATH: &str = "/var/log/capeos";
    /// Default runtime directory path (PID files, sockets, etc.).
    pub const DEFAULT_RUNTIME_PATH: &str = "/var/run/capeos";
    /// Default database directory path.
    pub const DEFAULT_DB_PATH: &str = "/var/lib/capeos/db";
}

/// The crate version from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

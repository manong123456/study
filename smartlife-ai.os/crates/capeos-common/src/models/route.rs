//! Route definitions for API gateway and service routing.

use serde::{Deserialize, Serialize};

/// A route mapping from a path prefix to a target service URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// The path prefix to match (e.g., `/api/v1`).
    pub path: String,
    /// The target service URL to forward requests to.
    pub target: String,
}

/// Request body for changing a service's listening port.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePortRequest {
    /// The new port number to listen on.
    pub port: u16,
}

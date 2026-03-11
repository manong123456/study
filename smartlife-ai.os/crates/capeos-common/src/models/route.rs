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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_serialization() {
        let route = Route {
            path: "/api/v1".to_string(),
            target: "http://127.0.0.1:8080".to_string(),
        };
        let json = serde_json::to_string(&route).unwrap();
        let deserialized: Route = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.path, route.path);
        assert_eq!(deserialized.target, route.target);
    }

    #[test]
    fn test_change_port_request() {
        let req = ChangePortRequest { port: 9090 };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: ChangePortRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.port, 9090);
    }
}

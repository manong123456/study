//! Dynamic routing table for the gateway reverse proxy.
//!
//! Maps path prefixes to backend target URLs. Used by the proxy to route incoming
//! requests and by the management API to register routes at runtime.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use capeos_common::models::Route;

/// Thread-safe, dynamic routing table mapping path prefixes to backend target URLs.
///
/// Supports concurrent reads and exclusive writes. Routes can be added at runtime
/// via the management API.
#[derive(Clone)]
pub struct RouteTable {
    inner: Arc<RwLock<HashMap<String, String>>>,
}

impl RouteTable {
    /// Creates a new empty route table.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a route mapping a path prefix to a target URL.
    ///
    /// # Arguments
    /// * `route` - The [`Route`] containing `path` (prefix) and `target` (backend URL).
    ///   Overwrites any existing route with the same path.
    pub async fn add_route(&self, route: Route) {
        let mut table = self.inner.write().await;
        tracing::info!("Registering route: {} -> {}", route.path, route.target);
        table.insert(route.path, route.target);
    }

    /// Finds the backend target for a request path using longest-prefix matching.
    ///
    /// If multiple routes match (e.g. `/api` and `/api/v1`), returns the target for the
    /// longest matching prefix.
    ///
    /// # Arguments
    /// * `path` - The request path (e.g. `/api/v1/users`).
    ///
    /// # Returns
    /// `Some(target_url)` if a matching route exists, `None` otherwise.
    pub async fn find_target(&self, path: &str) -> Option<String> {
        let table = self.inner.read().await;
        let mut best_match: Option<(&str, &str)> = None;
        for (prefix, target) in table.iter() {
            if path.starts_with(prefix.as_str()) {
                match best_match {
                    None => best_match = Some((prefix, target)),
                    Some((bp, _)) if prefix.len() > bp.len() => {
                        best_match = Some((prefix, target));
                    }
                    _ => {}
                }
            }
        }
        best_match.map(|(_, t)| t.to_string())
    }

    /// Returns all registered routes as a list.
    ///
    /// # Returns
    /// A [`Vec`] of [`Route`] entries (path, target) in no particular order.
    pub async fn list_routes(&self) -> Vec<Route> {
        let table = self.inner.read().await;
        table.iter().map(|(p, t)| Route {
            path: p.clone(),
            target: t.clone(),
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_table_is_empty() {
        let table = RouteTable::new();
        let routes = table.list_routes().await;
        assert!(routes.is_empty());
    }

    #[tokio::test]
    async fn test_add_and_find_route() {
        let table = RouteTable::new();
        table.add_route(Route {
            path: "/v1/sys".to_string(),
            target: "http://127.0.0.1:3000".to_string(),
        }).await;
        let target = table.find_target("/v1/sys/version").await;
        assert_eq!(target, Some("http://127.0.0.1:3000".to_string()));
    }

    #[tokio::test]
    async fn test_find_no_match() {
        let table = RouteTable::new();
        let target = table.find_target("/v1/sys").await;
        assert_eq!(target, None);
    }

    #[tokio::test]
    async fn test_longest_prefix_match() {
        let table = RouteTable::new();
        table.add_route(Route {
            path: "/v1".to_string(),
            target: "A".to_string(),
        }).await;
        table.add_route(Route {
            path: "/v1/sys".to_string(),
            target: "B".to_string(),
        }).await;
        let target = table.find_target("/v1/sys/version").await;
        assert_eq!(target, Some("B".to_string()));
    }

    #[tokio::test]
    async fn test_list_routes() {
        let table = RouteTable::new();
        table.add_route(Route {
            path: "/v1/sys".to_string(),
            target: "http://127.0.0.1:3000".to_string(),
        }).await;
        table.add_route(Route {
            path: "/v1/api".to_string(),
            target: "http://127.0.0.1:3001".to_string(),
        }).await;
        table.add_route(Route {
            path: "/v1/test".to_string(),
            target: "http://127.0.0.1:3002".to_string(),
        }).await;
        let routes = table.list_routes().await;
        assert_eq!(routes.len(), 3);
    }

    #[tokio::test]
    async fn test_overwrite_route() {
        let table = RouteTable::new();
        table.add_route(Route {
            path: "/v1/sys".to_string(),
            target: "http://127.0.0.1:3000".to_string(),
        }).await;
        table.add_route(Route {
            path: "/v1/sys".to_string(),
            target: "http://127.0.0.1:9999".to_string(),
        }).await;
        let target = table.find_target("/v1/sys/version").await;
        assert_eq!(target, Some("http://127.0.0.1:9999".to_string()));
    }
}

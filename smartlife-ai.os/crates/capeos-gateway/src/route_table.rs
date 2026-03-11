use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use capeos_common::models::Route;

#[derive(Clone)]
pub struct RouteTable {
    inner: Arc<RwLock<HashMap<String, String>>>,
}

impl RouteTable {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_route(&self, route: Route) {
        let mut table = self.inner.write().await;
        tracing::info!("Registering route: {} -> {}", route.path, route.target);
        table.insert(route.path, route.target);
    }

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

    pub async fn list_routes(&self) -> Vec<Route> {
        let table = self.inner.read().await;
        table.iter().map(|(p, t)| Route {
            path: p.clone(),
            target: t.clone(),
        }).collect()
    }
}

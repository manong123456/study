//! Management API for the gateway.
//!
//! Provides HTTP endpoints for route registration, listing routes, health checks,
//! and gateway port configuration. Intended for internal use (localhost only).

use crate::route_table::RouteTable;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use capeos_common::models::{route::ChangePortRequest, ApiResponse, Route};

/// Builds the management API router with route registration and port endpoints.
///
/// # Arguments
/// * `table` - Shared [`RouteTable`] used for route CRUD operations.
///
/// # Returns
/// An Axum [`Router`] with routes: `/ping`, `/v1/gateway/routes`, `/v1/gateway/port`.
pub fn management_router(table: RouteTable) -> Router {
    Router::new()
        .route("/v1/gateway/routes", post(create_route).get(list_routes))
        .route("/v1/gateway/port", get(get_port).put(change_port))
        .route("/ping", get(ping))
        .with_state(table)
}

/// Health check endpoint. Returns `"pong"` for liveness probes.
async fn ping() -> &'static str {
    "pong"
}

/// Registers a new route. Accepts a JSON [`Route`] and adds it to the route table.
async fn create_route(
    State(table): State<RouteTable>,
    Json(route): Json<Route>,
) -> Json<ApiResponse<Route>> {
    table.add_route(route.clone()).await;
    Json(ApiResponse::created(route))
}

/// Returns all registered routes as JSON.
async fn list_routes(State(table): State<RouteTable>) -> Json<ApiResponse<Vec<Route>>> {
    let routes = table.list_routes().await;
    Json(ApiResponse::ok(routes))
}

/// Returns the current gateway port (from `GATEWAY_PORT` env var, default 80).
async fn get_port() -> Json<ApiResponse<u16>> {
    let port: u16 = std::env::var("GATEWAY_PORT")
        .unwrap_or_else(|_| "80".to_string())
        .parse()
        .unwrap_or(80);
    Json(ApiResponse::ok(port))
}

/// Handles port change requests. Currently returns success; implementation is TODO.
async fn change_port(Json(_req): Json<ChangePortRequest>) -> Json<ApiResponse<()>> {
    // TODO: implement port change
    Json(ApiResponse::ok_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_ping() {
        let app = management_router(RouteTable::new());
        let req = Request::builder().uri("/ping").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.as_ref(), b"pong");
    }

    #[tokio::test]
    async fn test_create_and_list_routes() {
        let table = RouteTable::new();
        let app = management_router(table.clone());
        let body = serde_json::json!({"path": "/v1/test", "target": "http://127.0.0.1:5000"});
        let req = Request::builder()
            .method("POST")
            .uri("/v1/gateway/routes")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert!(resp.status().is_success());

        let req = Request::builder()
            .uri("/v1/gateway/routes")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let routes = json["data"].as_array().unwrap();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0]["path"], "/v1/test");
        assert_eq!(routes[0]["target"], "http://127.0.0.1:5000");
    }

    #[tokio::test]
    async fn test_get_port() {
        let app = management_router(RouteTable::new());
        let req = Request::builder()
            .uri("/v1/gateway/port")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert!(json["data"].is_number());
    }
}

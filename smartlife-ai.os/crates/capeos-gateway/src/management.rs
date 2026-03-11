//! Management API for the gateway.
//!
//! Provides HTTP endpoints for route registration, listing routes, health checks,
//! and gateway port configuration. Intended for internal use (localhost only).

use axum::{extract::State, routing::{get, post}, Json, Router};
use capeos_common::models::{ApiResponse, Route, route::ChangePortRequest};
use crate::route_table::RouteTable;

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
async fn list_routes(
    State(table): State<RouteTable>,
) -> Json<ApiResponse<Vec<Route>>> {
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

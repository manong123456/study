use axum::{extract::State, routing::{get, post}, Json, Router};
use capeos_common::models::{ApiResponse, Route, route::ChangePortRequest};
use crate::route_table::RouteTable;

pub fn management_router(table: RouteTable) -> Router {
    Router::new()
        .route("/v1/gateway/routes", post(create_route).get(list_routes))
        .route("/v1/gateway/port", get(get_port).put(change_port))
        .route("/ping", get(ping))
        .with_state(table)
}

async fn ping() -> &'static str {
    "pong"
}

async fn create_route(
    State(table): State<RouteTable>,
    Json(route): Json<Route>,
) -> Json<ApiResponse<Route>> {
    table.add_route(route.clone()).await;
    Json(ApiResponse::created(route))
}

async fn list_routes(
    State(table): State<RouteTable>,
) -> Json<ApiResponse<Vec<Route>>> {
    let routes = table.list_routes().await;
    Json(ApiResponse::ok(routes))
}

async fn get_port() -> Json<ApiResponse<u16>> {
    let port: u16 = std::env::var("GATEWAY_PORT")
        .unwrap_or_else(|_| "80".to_string())
        .parse()
        .unwrap_or(80);
    Json(ApiResponse::ok(port))
}

async fn change_port(Json(_req): Json<ChangePortRequest>) -> Json<ApiResponse<()>> {
    // TODO: implement port change
    Json(ApiResponse::ok_empty())
}

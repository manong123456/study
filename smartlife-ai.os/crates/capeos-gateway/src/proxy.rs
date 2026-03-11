use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    Router,
};
use crate::route_table::RouteTable;

pub fn proxy_router(table: RouteTable) -> Router {
    Router::new()
        .fallback(proxy_handler)
        .with_state(table)
}

async fn proxy_handler(
    State(table): State<RouteTable>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let path = req.uri().path().to_string();

    let target = table.find_target(&path).await
        .ok_or(StatusCode::NOT_FOUND)?;

    let target_uri = format!("{}{}", target.trim_end_matches('/'), req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or(&path));

    let client = reqwest::Client::new();
    let method = req.method().clone();
    let headers = req.headers().clone();

    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX).await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut builder = client.request(method, &target_uri);
    for (key, value) in headers.iter() {
        if key != "host" {
            builder = builder.header(key, value);
        }
    }

    let resp = builder.body(body_bytes).send().await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let resp_headers = resp.headers().clone();
    let resp_body = resp.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    let mut response = Response::builder().status(status);
    for (key, value) in resp_headers.iter() {
        response = response.header(key, value);
    }
    response.body(Body::from(resp_body)).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

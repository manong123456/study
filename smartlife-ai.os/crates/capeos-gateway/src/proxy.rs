//! Reverse proxy for the gateway.
//!
//! Forwards incoming HTTP requests to backend targets based on the route table.
//! Uses longest-prefix matching to select the target, then proxies the request
//! (method, headers, body) and returns the backend response.

use crate::route_table::RouteTable;
use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    Router,
};

/// Builds the proxy router. All requests fall through to [`proxy_handler`].
///
/// # Arguments
/// * `table` - Shared [`RouteTable`] for looking up backend targets by path.
///
/// # Returns
/// An Axum [`Router`] that proxies unmatched requests to backends.
pub fn proxy_router(table: RouteTable) -> Router {
    Router::new().fallback(proxy_handler).with_state(table)
}

/// Reverse-proxies the request to the backend target.
///
/// Looks up the target via [`RouteTable::find_target`] (longest-prefix match),
/// forwards the request (method, headers, body) to the target URL, and returns
/// the backend response. Returns 404 if no route matches, 502 on proxy errors.
async fn proxy_handler(
    State(table): State<RouteTable>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let path = req.uri().path().to_string();

    let target = table
        .find_target(&path)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    let target_uri = format!(
        "{}{}",
        target.trim_end_matches('/'),
        req.uri()
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or(&path)
    );

    let client = reqwest::Client::new();
    let method = req.method().clone();
    let headers = req.headers().clone();

    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut builder = client.request(method, &target_uri);
    for (key, value) in headers.iter() {
        if key != "host" {
            builder = builder.header(key, value);
        }
    }

    let resp = builder
        .body(body_bytes)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let status =
        StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let resp_headers = resp.headers().clone();
    let resp_body = resp.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    let mut response = Response::builder().status(status);
    for (key, value) in resp_headers.iter() {
        response = response.header(key, value);
    }
    response
        .body(Body::from(resp_body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_proxy_returns_404_for_unknown_route() {
        let app = proxy_router(RouteTable::new());
        let req = Request::builder()
            .uri("/unknown/path")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}

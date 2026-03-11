//! Integration tests for the CapeOS gateway route registration and proxy flow.

use capeos_common::models::Route;

#[test]
fn test_route_model_roundtrip() {
    let route = Route {
        path: "/v1/sys".to_string(),
        target: "http://127.0.0.1:3000".to_string(),
    };
    let json = serde_json::to_string(&route).unwrap();
    let parsed: Route = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.path, "/v1/sys");
    assert_eq!(parsed.target, "http://127.0.0.1:3000");
}

#[test]
fn test_multiple_routes_serialization() {
    let routes = vec![
        Route {
            path: "/v1/sys".to_string(),
            target: "http://127.0.0.1:3000".to_string(),
        },
        Route {
            path: "/v1/file".to_string(),
            target: "http://127.0.0.1:3001".to_string(),
        },
    ];
    let json = serde_json::to_string(&routes).unwrap();
    let parsed: Vec<Route> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.len(), 2);
}

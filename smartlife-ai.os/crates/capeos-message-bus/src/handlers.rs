//! HTTP and WebSocket handlers for the message bus API.
//!
//! Endpoints for listing/registering event types, publishing events, and
//! subscribing via WebSocket.

use axum::{
    extract::{Path, State, WebSocketUpgrade, ws::{Message, WebSocket}},
    Json,
    response::IntoResponse,
};
use capeos_common::models::ApiResponse;
use capeos_common::models::event::{EventType, Event};
use crate::store::BusState;

/// Lists all registered event types.
///
/// Returns a JSON array of [`EventType`] entries from the registry.
pub async fn list_event_types(
    State(state): State<BusState>,
) -> Json<ApiResponse<Vec<EventType>>> {
    let types: Vec<EventType> = state.event_types.iter().map(|e| e.value().clone()).collect();
    Json(ApiResponse::ok(types))
}

/// Registers one or more event types.
///
/// Accepts a JSON array of [`EventType`]. Each is stored with key `source_id:name`.
/// Overwrites existing entries with the same key.
pub async fn register_event_types(
    State(state): State<BusState>,
    Json(types): Json<Vec<EventType>>,
) -> Json<ApiResponse<()>> {
    for et in types {
        let key = format!("{}:{}", et.source_id, et.name);
        state.event_types.insert(key, et);
    }
    Json(ApiResponse::ok_empty())
}

/// Publishes an event to all subscribers.
///
/// Path params: `source_id`, `name`. Body: JSON object with event `properties`.
/// Creates an [`Event`] with current timestamp and broadcasts it via the channel.
pub async fn publish_event(
    State(state): State<BusState>,
    Path((source_id, name)): Path<(String, String)>,
    Json(properties): Json<serde_json::Value>,
) -> Json<ApiResponse<()>> {
    let event = Event {
        source_id,
        name,
        properties,
        timestamp: chrono_now(),
    };
    let _ = state.sender.send(event);
    Json(ApiResponse::ok_empty())
}

/// WebSocket upgrade handler for event subscriptions.
///
/// Upgrades the connection and spawns [`handle_ws`] to stream events as JSON
/// text messages to the client.
pub async fn subscribe(
    State(state): State<BusState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

/// Handles WebSocket connection: subscribes to the broadcast channel and forwards
/// each event as a JSON text message. Closes when the channel is dropped or send fails.
async fn handle_ws(mut socket: WebSocket, state: BusState) {
    let mut rx = state.sender.subscribe();
    while let Ok(event) = rx.recv().await {
        if let Ok(json) = serde_json::to_string(&event) {
            if socket.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    }
}

fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}

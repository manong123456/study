//! In-memory state for the message bus.
//!
//! Holds the event type registry and broadcast channel for pub/sub.

use capeos_common::models::event::{Event, EventType};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Shared state for the message bus service.
///
/// Combines an event type registry with a broadcast channel for delivering
/// events to all subscribers.
#[derive(Clone)]
pub struct BusState {
    /// Registry of event types (key: `source_id:name`). Used to validate and list
    /// registered event types.
    pub event_types: Arc<DashMap<String, EventType>>,
    /// Broadcast channel for publishing events. Subscribers receive events in real time.
    pub sender: broadcast::Sender<Event>,
}

impl BusState {
    /// Creates new bus state with an empty event type registry and a broadcast channel (capacity 1024).
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self {
            event_types: Arc::new(DashMap::new()),
            sender,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use capeos_common::models::event::Event;

    #[test]
    fn test_new_bus_state() {
        let state = BusState::new();
        assert!(state.event_types.is_empty());
    }

    #[tokio::test]
    async fn test_broadcast_send_receive() {
        let state = BusState::new();
        let mut rx = state.sender.subscribe();
        let event = Event {
            source_id: "test".to_string(),
            name: "test_event".to_string(),
            properties: serde_json::json!({"key": "value"}),
            timestamp: "12345".to_string(),
        };
        let _ = state.sender.send(event.clone());
        let received = rx.recv().await.unwrap();
        assert_eq!(received.source_id, event.source_id);
        assert_eq!(received.name, event.name);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let state = BusState::new();
        let mut rx1 = state.sender.subscribe();
        let mut rx2 = state.sender.subscribe();
        let event = Event {
            source_id: "src".to_string(),
            name: "evt".to_string(),
            properties: serde_json::json!({}),
            timestamp: "0".to_string(),
        };
        let _ = state.sender.send(event.clone());
        let r1 = rx1.recv().await.unwrap();
        let r2 = rx2.recv().await.unwrap();
        assert_eq!(r1.source_id, r2.source_id);
        assert_eq!(r1.name, r2.name);
    }
}

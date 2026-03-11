//! In-memory state for the message bus.
//!
//! Holds the event type registry and broadcast channel for pub/sub.

use std::sync::Arc;
use tokio::sync::broadcast;
use dashmap::DashMap;
use capeos_common::models::event::{EventType, Event};

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

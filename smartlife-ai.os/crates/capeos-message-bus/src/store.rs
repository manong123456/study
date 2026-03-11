use std::sync::Arc;
use tokio::sync::broadcast;
use dashmap::DashMap;
use capeos_common::models::event::{EventType, Event};

#[derive(Clone)]
pub struct BusState {
    pub event_types: Arc<DashMap<String, EventType>>,
    pub sender: broadcast::Sender<Event>,
}

impl BusState {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self {
            event_types: Arc::new(DashMap::new()),
            sender,
        }
    }
}

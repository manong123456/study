//! Event and property type definitions for event-driven workflows.

use serde::{Deserialize, Serialize};

/// Schema definition for an event type, including its properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventType {
    /// The event type name.
    pub name: String,
    /// Identifier of the source that emits this event.
    pub source_id: String,
    /// List of property definitions for this event type.
    pub properties: Vec<PropertyType>,
}

/// Schema for a single property within an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyType {
    /// The property name.
    pub name: String,
    /// Optional human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// An event instance with payload and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Identifier of the source that emitted this event.
    pub source_id: String,
    /// The event type name.
    pub name: String,
    /// JSON object containing the event payload/properties.
    pub properties: serde_json::Value,
    /// ISO 8601 timestamp when the event occurred.
    pub timestamp: String,
}

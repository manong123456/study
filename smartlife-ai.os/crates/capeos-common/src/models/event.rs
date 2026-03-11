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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_serialization() {
        let event_type = EventType {
            name: "user.created".to_string(),
            source_id: "user-service".to_string(),
            properties: vec![
                PropertyType {
                    name: "user_id".to_string(),
                    description: Some("User identifier".to_string()),
                },
                PropertyType {
                    name: "email".to_string(),
                    description: None,
                },
            ],
        };
        let json = serde_json::to_string(&event_type).unwrap();
        let deserialized: EventType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, event_type.name);
        assert_eq!(deserialized.source_id, event_type.source_id);
        assert_eq!(deserialized.properties.len(), 2);
    }

    #[test]
    fn test_event_serialization() {
        let event = Event {
            source_id: "user-service".to_string(),
            name: "user.created".to_string(),
            properties: serde_json::json!({"user_id": 123, "email": "a@b.com"}),
            timestamp: "2025-03-11T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.source_id, event.source_id);
        assert_eq!(deserialized.name, event.name);
        assert_eq!(deserialized.properties, event.properties);
        assert_eq!(deserialized.timestamp, event.timestamp);
    }
}

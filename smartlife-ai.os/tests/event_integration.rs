//! Integration tests for the CapeOS event system models.

use capeos_common::models::event::{Event, EventType, PropertyType};

#[test]
fn test_event_type_with_properties() {
    let et = EventType {
        name: "capeos:system:utilization".to_string(),
        source_id: "capeos".to_string(),
        properties: vec![
            PropertyType {
                name: "cpu_percent".to_string(),
                description: Some("CPU usage percentage".to_string()),
            },
            PropertyType {
                name: "memory_used".to_string(),
                description: None,
            },
        ],
    };
    let json = serde_json::to_string(&et).unwrap();
    assert!(json.contains("cpu_percent"));
    assert!(json.contains("capeos:system:utilization"));
}

#[test]
fn test_event_with_json_properties() {
    let event = Event {
        source_id: "capeos".to_string(),
        name: "capeos:file:operate".to_string(),
        properties: serde_json::json!({
            "operation": "copy",
            "progress": 75,
            "source": "/data/file.txt",
            "destination": "/backup/file.txt"
        }),
        timestamp: "1710000000".to_string(),
    };
    let json = serde_json::to_string(&event).unwrap();
    let parsed: Event = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.source_id, "capeos");
    assert_eq!(parsed.properties["progress"], 75);
}

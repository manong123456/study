use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventType {
    pub name: String,
    pub source_id: String,
    pub properties: Vec<PropertyType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyType {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub source_id: String,
    pub name: String,
    pub properties: serde_json::Value,
    pub timestamp: String,
}

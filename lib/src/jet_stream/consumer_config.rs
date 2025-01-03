use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ConsumerConfig {
    pub durable_name: String,
    pub filter_subject: Option<String>,
    pub filter_subjects: Option<Vec<String>>,
}

impl ConsumerConfig {
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

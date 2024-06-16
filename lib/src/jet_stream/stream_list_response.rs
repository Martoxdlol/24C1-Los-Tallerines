use serde::{Deserialize, Serialize};

use super::stream_info::StreamInfo;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct JetStreamStreamListResponse {
    pub r#type: String,
    pub total: i32,
    pub limit: i32,
    pub streams: Vec<StreamInfo>,
}

impl JetStreamStreamListResponse {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ParametrosInfo {
    pub auth_required: Option<bool>,
    pub max_payload: Option<u64>,
}

impl ParametrosInfo {
    pub fn from_json(json: &str) -> Result<ParametrosInfo> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
    }
}

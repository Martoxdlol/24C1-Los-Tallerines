use serde::{Deserialize, Serialize};

use super::consumer_info::ConsumerInfo;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct JetStreamConsumerListaRespuesta {
    pub r#type: String,
    pub total: i32,
    pub limit: i32,
    pub consumers: Vec<ConsumerInfo>,
}

impl JetStreamConsumerListaRespuesta {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

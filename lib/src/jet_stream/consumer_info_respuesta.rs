use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::consumer_config::ConsumerConfig;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct JSConsumerInfoRespuesta {
    pub r#type: String,
    pub config: ConsumerConfig,
    pub created: String,
    pub ts: String,
}

impl JSConsumerInfoRespuesta {
    pub fn new(config: ConsumerConfig) -> Self {
        Self {
            r#type: "io.nats.jetstream.api.v1.consumer_info_response".to_string(),
            config,
            created: Utc::now().to_rfc3339(),
            ts: Utc::now().to_rfc3339(),
        }
    }

    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

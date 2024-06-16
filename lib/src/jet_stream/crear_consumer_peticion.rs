use serde::{Deserialize, Serialize};

use super::consumer_config::ConsumerConfig;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct JSPeticionCrearConsumer {
    pub config: ConsumerConfig,
}

impl JSPeticionCrearConsumer {
    pub fn new(config: ConsumerConfig) -> Self {
        Self { config }
    }

    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

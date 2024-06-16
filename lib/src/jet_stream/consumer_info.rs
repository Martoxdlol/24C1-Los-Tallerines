use serde::{Deserialize, Serialize};

use super::consumer_config::ConsumerConfig;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ConsumerInfo {
    pub config: ConsumerConfig,
    pub created: String,
    pub ts: String,
}

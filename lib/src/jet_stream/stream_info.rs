use serde::{Deserialize, Serialize};

use super::{stream_config::StreamConfig, stream_state::JetStreamStreamState};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct StreamInfo {
    pub config: StreamConfig,
    pub created: String,
    pub state: JetStreamStreamState,
    pub ts: String,
}

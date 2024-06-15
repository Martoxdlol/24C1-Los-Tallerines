use serde::{Deserialize, Serialize};

use super::{stream_config::StreamConfig, stream_state::JetStreamStreamState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JetStreamStreamInfo {
    config: StreamConfig,
    created: String,
    state: JetStreamStreamState,
    ts: String,
}

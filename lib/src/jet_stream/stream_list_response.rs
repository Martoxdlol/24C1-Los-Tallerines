use serde::{Deserialize, Serialize};

use super::stream_info::JetStreamStreamInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JetStreamStreamListResponse {
    r#type: String,
    total: u64,
    limit: u64,
    streams: Vec<JetStreamStreamInfo>,
}

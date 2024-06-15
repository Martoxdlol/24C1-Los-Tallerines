use serde::{Deserialize, Serialize};

use super::stream_info::StreamInfo;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct JetStreamStreamListResponse {
    r#type: String,
    total: u64,
    limit: u64,
    streams: Vec<StreamInfo>,
}

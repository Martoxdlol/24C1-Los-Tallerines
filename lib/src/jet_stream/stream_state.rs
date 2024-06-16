use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct JetStreamStreamState {
    messages: u64,
    bytes: u64,
    first_seq: u64,
    first_ts: String,
    last_seq: u64,
    last_ts: String,
    consumer_count: u64,
}

impl JetStreamStreamState {
    pub fn new() -> Self {
        JetStreamStreamState {
            messages: 0,
            bytes: 0,
            first_seq: 0,
            first_ts: Utc::now().to_rfc3339(),
            last_seq: 0,
            last_ts: Utc::now().to_rfc3339(),
            consumer_count: 0,
        }
    }
}

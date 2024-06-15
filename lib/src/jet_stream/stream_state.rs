use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JetStreamStreamState {
    messages: u64,
    bytes: u64,
    first_seq: u64,
    first_ts: String,
    last_seq: u64,
    last_ts: String,
    consumer_count: u64,
}

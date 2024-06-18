use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::{stream_config::StreamConfig, stream_state::JetStreamStreamState};
// {
//     "type": "io.nats.jetstream.api.v1.stream_create_response",
//     "config": {
//       "name": "asdasd",
//       "subjects": [
//         "asd"
//       ],
//       "retention": "limits",
//       "max_consumers": -1,
//       "max_msgs": -1,
//       "max_bytes": -1,
//       "max_age": 0,
//       "max_msgs_per_subject": -1,
//       "max_msg_size": -1,
//       "discard": "old",
//       "storage": "file",
//       "num_replicas": 1,
//       "duplicate_window": 120000000000,
//       "compression": "none",
//       "allow_direct": true,
//       "mirror_direct": false,
//       "sealed": false,
//       "deny_delete": false,
//       "deny_purge": false,
//       "allow_rollup_hdrs": false,
//       "consumer_limits": {}
//     },
//     "created": "2024-06-16T01:16:32.045107135Z",
//     "state": {
//       "messages": 0,
//       "bytes": 0,
//       "first_seq": 1,
//       "first_ts": "1970-01-01T00:00:00Z",
//       "last_seq": 0,
//       "last_ts": "0001-01-01T00:00:00Z",
//       "consumer_count": 0
//     },
//     "ts": "2024-06-16T01:17:29.556873303Z",
//     "did_create": true
//   }

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct JSCrearStreamRespuesta {
    pub r#type: String,
    pub config: StreamConfig,
    pub created: String,
    pub state: JetStreamStreamState,
    pub ts: String,
    pub did_create: bool,
}

impl JSCrearStreamRespuesta {
    pub fn new(config: StreamConfig, se_creo: bool) -> Self {
        Self {
            r#type: "io.nats.jetstream.api.v1.stream_create_response".to_string(),
            config,
            created: Utc::now().to_rfc3339(),
            state: JetStreamStreamState::new(),
            ts: Utc::now().to_rfc3339(),
            did_create: se_creo,
        }
    }

    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

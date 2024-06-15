use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct JSApiInfoResponse {
    pub r#type: String,
    pub memory: i32,
    pub storage: i32,
    pub reserved_memory: i32,
    pub reserved_storage: i32,
    pub streams: i32,
    pub consumers: i32,
    pub limits: JSApiInfoLimits,
    pub api: JSApiInfoApi,
}

impl JSApiInfoResponse {
    pub fn new(streams: i32, consumers: i32) -> Self {
        JSApiInfoResponse {
            r#type: "io.nats.jetstream.api.v1.account_info_response".to_string(),
            memory: 0,
            storage: 0,
            reserved_memory: 0,
            reserved_storage: 0,
            streams,
            consumers,
            api: JSApiInfoApi {
                errors: 0,
                total: 0,
            },
            limits: JSApiInfoLimits::new(),
        }
    }

    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct JSApiInfoLimits {
    pub max_memory: i32,
    pub max_storage: i32,
    pub max_streams: i32,
    pub max_consumers: i32,
    pub max_ack_pending: i32,
    pub memory_max_stream_bytes: i32,
    pub storage_max_stream_bytes: i32,
    pub max_bytes_required: bool,
}

impl JSApiInfoLimits {
    pub fn new() -> Self {
        JSApiInfoLimits {
            max_memory: -1,
            max_storage: -1,
            max_streams: -1,
            max_consumers: -1,
            max_ack_pending: -1,
            memory_max_stream_bytes: -1,
            storage_max_stream_bytes: -1,
            max_bytes_required: false,
        }
    }

    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct JSApiInfoApi {
    pub total: i32,
    pub errors: i32,
}

impl JSApiInfoApi {
    pub fn new() -> Self {
        Self {
            total: -1,
            errors: -1,
        }
    }

    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

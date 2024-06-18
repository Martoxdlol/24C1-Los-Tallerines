use serde::{Deserialize, Serialize};
// {"type":"io.nats.jetstream.api.v1.stream_names_response","total":1,"offset":0,"limit":1024,"streams":["valeria"]}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct JSNombresConsumersRespuesta {
    pub r#type: String,
    pub total: i32,
    pub offset: i32,
    pub limit: i32,
    pub consumers: Vec<String>,
}

impl JSNombresConsumersRespuesta {
    pub fn new(streams: Vec<String>) -> Self {
        Self {
            r#type: "io.nats.jetstream.api.v1.consumer_names_response".to_string(),
            total: streams.len() as i32,
            offset: 0,
            limit: (streams.len() + 1) as i32,
            consumers: streams,
        }
    }

    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

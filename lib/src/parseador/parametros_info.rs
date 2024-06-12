use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
/// Parámetros para la información del servidor NATS
pub struct ParametrosInfo {
    pub auth_required: Option<bool>,
}

impl ParametrosInfo {
    /// Forma la estructura desde un json
    pub fn from_json(json: &str) -> Result<ParametrosInfo> {
        serde_json::from_str(json)
    }

    /// Forma un json desde la estructura
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
    }
}

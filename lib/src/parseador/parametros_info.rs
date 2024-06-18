use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
/// Parámetros para la información del servidor NATS
pub struct ParametrosInfo {
    pub auth_required: Option<bool>,
    pub max_payload: Option<u64>,
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

#[cfg(test)]

mod tests {
    use super::ParametrosInfo;

    #[test]
    fn crear_parametros() {
        let parametros = ParametrosInfo {
            auth_required: Some(true),
        };
        assert_eq!(parametros.auth_required, Some(true));
    }

    #[test]
    fn crear_json() {
        let parametros = ParametrosInfo {
            auth_required: Some(true),
        };
        let json = parametros.to_json().unwrap();
        assert_eq!(json, "{\"auth_required\":true}");
    }

    #[test]
    fn conseguir_info() {
        let json = "{\"auth_required\":true}";
        let parametros = ParametrosInfo::from_json(json).unwrap();
        assert_eq!(parametros.auth_required, Some(true));
    }

    #[test]
    fn conseguir_info_vacio() {
        let json = "{}";
        let parametros = ParametrosInfo::from_json(json).unwrap();
        assert_eq!(parametros.auth_required, None);
    }
}

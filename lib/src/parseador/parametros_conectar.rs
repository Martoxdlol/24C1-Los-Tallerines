use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
/// Parámetros para conectar al servidor NATS. La autenticación
pub struct ParametrosConectar {
    pub user: Option<String>,
    pub pass: Option<String>,
    pub verbose: Option<bool>,
}

impl ParametrosConectar {
    /// El new de la estructura
    pub fn user_pass(user: &str, pass: &str) -> Self {
        Self {
            user: Some(user.to_string()),
            pass: Some(pass.to_string()),
            verbose: None,
        }
    }

    /// Genera la estructura desde un json
    pub fn from_json(json: &str) -> Result<ParametrosConectar> {
        serde_json::from_str(json)
    }

    /// Genera un json desde la estructura
    pub fn to_json(&self) -> String {
        if let Ok(txt) = serde_json::to_string(self) {
            return txt;
        }

        "{}".to_string()
    }

    /// Devuelve lo que se ingreso en user como string
    pub fn user_str(&self) -> String {
        match &self.user {
            Some(user) => user.to_string(),
            None => "".to_string(),
        }
    }

    /// Devuelve lo que se ingreso en pass como string
    pub fn pass_str(&self) -> String {
        match &self.pass {
            Some(pass) => pass.to_string(),
            None => "".to_string(),
        }
    }
}

#[cfg(test)]

mod tests {
    use super::ParametrosConectar;

    #[test]
    fn crear_parametros() {
        let parametros = ParametrosConectar::user_pass("usuario", "contraseña");
        assert_eq!(parametros.user_str(), "usuario");
        assert_eq!(parametros.pass_str(), "contraseña");
    }

    #[test]
    fn generar_json() {
        let parametros = ParametrosConectar::user_pass("usuario", "contraseña");
        let json = parametros.to_json();
        assert_eq!(
            json,
            "{\"user\":\"usuario\",\"pass\":\"contraseña\",\"verbose\":null}"
        );
    }

    #[test]
    fn conseguir_parametros_por_json() {
        let json = "{\"user\":\"usuario\",\"pass\":\"contraseña\"}";
        let parametros = ParametrosConectar::from_json(json).unwrap();
        assert_eq!(parametros.user_str(), "usuario");
        assert_eq!(parametros.pass_str(), "contraseña");
    }

    #[test]
    fn conseguir_parametros_por_json_vacio() {
        let json = "{}";
        let parametros = ParametrosConectar::from_json(json).unwrap();
        assert_eq!(parametros.user_str(), "");
        assert_eq!(parametros.pass_str(), "");
    }
}

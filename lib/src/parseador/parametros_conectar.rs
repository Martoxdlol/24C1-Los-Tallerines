use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ParametrosConectar {
    pub user: Option<String>,
    pub pass: Option<String>,
    pub verbose: Option<bool>,
}

impl ParametrosConectar {
    pub fn user_pass(user: &str, pass: &str) -> Self {
        Self {
            user: Some(user.to_string()),
            pass: Some(pass.to_string()),
            verbose: None,
        }
    }

    pub fn from_json(json: &str) -> Result<ParametrosConectar> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> String {
        if let Ok(txt) = serde_json::to_string(self) {
            return txt;
        }

        "{}".to_string()
    }

    pub fn user_str(&self) -> String {
        match &self.user {
            Some(user) => user.to_string(),
            None => "".to_string(),
        }
    }

    pub fn pass_str(&self) -> String {
        match &self.pass {
            Some(pass) => pass.to_string(),
            None => "".to_string(),
        }
    }
}

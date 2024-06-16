use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct StreamConfig {
    /// Un nombre para el Stream. No debe tener espacios, tabulaciones ni caracteres de punto `.`
    pub name: String,
    /// Cuánto puede crecer el Stream en bytes totales antes de que se active la política de descarte configurada
    pub max_bytes: i64,
    /// Cuánto puede crecer el Stream en mensajes totales antes de que se active la política de descarte configurada
    pub max_msgs: i64,
    /// Qué sujetos NATS poblarán este stream. Soporta comodines. Predeterminado solo al
    /// nombre del stream configurado.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subjects: Vec<String>,
    /// Cuántos Consumidores se pueden definir para un Stream dado, -1 para ilimitado
    pub max_consumers: i32,
    /// Edad máxima de cualquier mensaje en el stream, expresada en nanosegundos
    #[serde(with = "serde_nanos")]
    pub max_age: Duration,
    /// El mensaje más grande que será aceptado por el Stream
    pub max_msg_size: i32,
    /// No me importa, per el CLI de nats lo requiere
    pub num_replicas: i32,
}

impl StreamConfig {
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

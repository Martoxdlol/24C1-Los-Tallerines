use crate::{csv::{csv_encodear_linea, csv_parsear_linea}, serializables::{error::DeserializationError, Serializable}};

use std::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct Bateria {
    pub duracion_bateria: u64, // En segundos
    pub bateria_minima: u64,
    pub canal_notificar_bateria: Option<Sender<u64>>,
}

impl Bateria {
    pub fn new(duracion_bateria: u64, bateria_minima: u64, canal_notificar_bateria: Sender<u64>) -> Self {
        Bateria {
            duracion_bateria,
            bateria_minima,
            canal_notificar_bateria: Some(canal_notificar_bateria),
        }
    }
}

impl Serializable for Bateria {
    fn serializar(&self) -> Vec<u8> {
        let mut parametros: Vec<String> = Vec::new();
        parametros.push(format!("{}", self.duracion_bateria));
        parametros.push(format!("{}", self.bateria_minima));
        csv_encodear_linea(&parametros).into_bytes()
    }

    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError> {
        let linea =
            String::from_utf8(data.to_vec()).map_err(|_| DeserializationError::InvalidData)?;
        let mut parametros = csv_parsear_linea(linea.as_str()).into_iter();

        let duracion_bateria = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;
        let bateria_minima = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        Ok(Bateria {
            duracion_bateria, 
            bateria_minima,
            canal_notificar_bateria: None,
        })
    }
}
use crate::csv::{csv_encodear_linea, csv_parsear_linea};

use super::{error::DeserializationError, Serializable};

pub struct CamaraSerializable {
    pub id: u64,
    pub lat: f64,
    pub lon: f64,
    pub rango: f64,
}

impl Serializable for CamaraSerializable {
    fn serializar(&self) -> Vec<u8> {
        let mut parametros: Vec<String> = Vec::new();
        parametros.push(format!("{}", self.id));
        parametros.push(format!("{}", self.lat));
        parametros.push(format!("{}", self.lon));
        parametros.push(format!("{}", self.rango));
        csv_encodear_linea(&parametros).into_bytes()
    }

    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError> {
        let linea =
            String::from_utf8(data.to_vec()).map_err(|_| DeserializationError::InvalidData)?;
        let mut parametros = csv_parsear_linea(linea.as_str()).into_iter();

        let id = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;
        let lat = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;
        let lon = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;
        let rango = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        Ok(CamaraSerializable {
            id,
            lat,
            lon,
            rango,
        })
    }
}

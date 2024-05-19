use std::collections::HashSet;

use crate::{
    coordenadas::Coordenadas,
    csv::{csv_encodear_linea, csv_parsear_linea},
    serializables::{error::DeserializationError, Serializable},
};

#[derive(Debug, Clone)]
pub struct Camara {
    pub id: u64,
    pub lat: f64,
    pub lon: f64,
    pub rango: f64,
    /// Incidentes atendidos por cada camara.
    pub incidentes_primarios: HashSet<u64>,
    /// Incidentes que atiende cada cámara por ser lindante
    pub incidentes_secundarios: HashSet<u64>,
}

impl Camara {
    pub fn new(id: u64, lat: f64, lon: f64, rango: f64) -> Self {
        Camara {
            id,
            lat,
            lon,
            rango,
            incidentes_primarios: HashSet::new(),
            incidentes_secundarios: HashSet::new(),
        }
    }

    pub fn activa(&self) -> bool {
        self.incidentes_primarios.len() > 0 || self.incidentes_secundarios.len() > 0
    }

    pub fn posicion(&self) -> Coordenadas {
        Coordenadas::from_lat_lon(self.lat, self.lon)
    }
}

impl Serializable for Camara {
    fn serializar(&self) -> Vec<u8> {
        let mut parametros: Vec<String> = Vec::new();
        parametros.push(format!("{}", self.id));
        parametros.push(format!("{}", self.lat));
        parametros.push(format!("{}", self.lon));
        parametros.push(format!("{}", self.rango));
        parametros.push(format!(
            "{}",
            serializar_vector_incidentes(&self.incidentes_primarios)
        ));
        parametros.push(format!(
            "{}",
            serializar_vector_incidentes(&self.incidentes_secundarios)
        ));
        csv_encodear_linea(&parametros).into_bytes()
    }

    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError> {
        let linea =
            String::from_utf8(data.to_vec()).map_err(|_| DeserializationError::InvalidData)?;
        let mut parametros = csv_parsear_linea(linea.as_str()).into_iter();

        let id = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;
        let lat = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;
        let lon = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;
        let rango = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;
        let incidentes_primarios = deserialize_vector_incidentes(
            &parametros
                .next()
                .ok_or(DeserializationError::MissingField)?,
        )?;
        let incidentes_secundarios = deserialize_vector_incidentes(
            &parametros
                .next()
                .ok_or(DeserializationError::MissingField)?,
        )?;

        Ok(Camara {
            id,
            lat,
            lon,
            rango,
            incidentes_primarios,
            incidentes_secundarios,
        })
    }
}

fn serializar_vector_incidentes(incidentes: &HashSet<u64>) -> String {
    incidentes
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(";")
}

fn deserialize_vector_incidentes(data: &str) -> Result<HashSet<u64>, DeserializationError> {
    data.split(";")
        .map(|id| id.parse().map_err(|_| DeserializationError::InvalidData))
        .collect()
}

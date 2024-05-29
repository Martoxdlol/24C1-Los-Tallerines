use std::collections::HashSet;

use crate::{
    coordenadas::Coordenadas,
    csv::{csv_encodear_linea, csv_parsear_linea},
    serializables::{error::DeserializationError, Serializable},
};

#[derive(Debug, Clone, PartialEq)]
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
        !self.incidentes_primarios.is_empty() || !self.incidentes_secundarios.is_empty()
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
        parametros.push(serializar_vector_incidentes(&self.incidentes_primarios).to_string());
        parametros.push(serializar_vector_incidentes(&self.incidentes_secundarios).to_string());
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
    if data.trim().is_empty() {
        return Ok(HashSet::new());
    }

    data.split(';')
        .map(|id| id.parse().map_err(|_| DeserializationError::InvalidData))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializar_camara() {
        let camara = Camara {
            id: 1,
            lat: 1.0,
            lon: 2.0,
            rango: 3.0,
            incidentes_primarios: vec![1, 2, 3].into_iter().collect(),
            incidentes_secundarios: vec![4, 5, 6].into_iter().collect(),
        };

        let serializado = camara.serializar();
        let deserializado = Camara::deserializar(&serializado).unwrap();

        assert_eq!(camara, deserializado);
    }

    #[test]
    fn serializar_sin_incidentes() {
        let camara = Camara {
            id: 1,
            lat: 1.0,
            lon: 2.0,
            rango: 3.0,
            incidentes_primarios: HashSet::new(),
            incidentes_secundarios: HashSet::new(),
        };

        let serializado = camara.serializar();
        let deserializado = Camara::deserializar(&serializado).unwrap();

        assert_eq!(camara, deserializado);
    }
}

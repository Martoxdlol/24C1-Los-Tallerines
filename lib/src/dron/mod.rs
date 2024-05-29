use std::collections::HashSet;
use std::str::FromStr;

use crate::{
    coordenadas::Coordenadas,
    serializables::{Serializable, error::DeserializationError},
    csv::{csv_encodear_linea, csv_parsear_linea},
};

#[derive(Debug, Clone)]
pub enum EstadoDron {
    EnEspera,
    VolviendoACentroDeOperacion,
    YendoAIncidente,
    AtendiendoIncidente,
    YendoACentral,
    CargandoEnCentral,
}

#[derive(Debug, Clone)]
pub struct Dron {
    pub id: u64,
    pub latitud: f64,
    pub longitud: f64,
    pub rango: f64,
    estado: EstadoDron,
    direccion: f64, // En grados, sentido horario, empezando desde el norte
    velocidad: f64,
    duracion_bateria: u64, // En segundos
    pub incidentes_cercanos: HashSet<u64>,
    latitud_central: f64,
    longitud_central: f64,
    latitud_centro_operaciones: f64,
    longitud_centro_operaciones: f64,
}


impl Dron {
    pub fn new(id: u64, latitud: f64, longitud: f64, rango: f64, estado: EstadoDron, direccion: f64, velocidad: f64, duracion_bateria: u64, longitud_central: f64, latitud_central: f64, latitud_centro_operaciones: f64, longitud_centro_operaciones: f64) -> Self {
        Dron {
            id,
            latitud,
            longitud,
            rango,
            estado,
            direccion,
            velocidad,
            duracion_bateria, 
            incidentes_cercanos: HashSet::new(),
            latitud_central, 
            longitud_central,
            latitud_centro_operaciones,
            longitud_centro_operaciones,
        }
    }

    pub fn posicion(&self) -> Coordenadas {
        Coordenadas::from_lat_lon(self.latitud, self.longitud)
    }
}

impl Serializable for Dron {

    fn serializar(&self) -> Vec<u8> {
        let mut parametros: Vec<String> = Vec::new();
        parametros.push(format!("{}", self.id));
        parametros.push(format!("{}", self.latitud));
        parametros.push(format!("{}", self.longitud));
        parametros.push(format!("{}", self.rango));
        parametros.push(format!("{:?}", self.estado));
        parametros.push(format!("{}", self.direccion));
        parametros.push(format!("{}", self.velocidad));
        parametros.push(format!("{}", self.duracion_bateria));
        parametros.push(serializar_vector_incidentes(&self.incidentes_cercanos).to_string());
        parametros.push(format!("{}", self.latitud_central));
        parametros.push(format!("{}", self.longitud_central));
        parametros.push(format!("{}", self.latitud_centro_operaciones));
        parametros.push(format!("{}", self.longitud_centro_operaciones));

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

        let latitud = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let longitud = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let rango = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let estado = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let direccion = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let velocidad = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let duracion_bateria = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let incidentes_cercanos = deserialize_vector_incidentes(
            &parametros
                .next()
                .ok_or(DeserializationError::MissingField)?,
        )?;

        let latitud_central = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let longitud_central = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let latitud_centro_operaciones = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let longitud_centro_operaciones = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        Ok(Dron {
            id,
            latitud,
            longitud,
            rango,
            estado,
            direccion,
            velocidad,
            duracion_bateria,
            incidentes_cercanos,
            latitud_central,
            longitud_central,
            latitud_centro_operaciones,
            longitud_centro_operaciones,
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

impl FromStr for EstadoDron {
    type Err = DeserializationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EnEspera" => Ok(EstadoDron::EnEspera),
            "VolviendoACentroDeOperacion" => Ok(EstadoDron::VolviendoACentroDeOperacion),
            "YendoAIncidente" => Ok(EstadoDron::YendoAIncidente),
            "AtendiendoIncidente" => Ok(EstadoDron::AtendiendoIncidente),
            "YendoACentral" => Ok(EstadoDron::YendoACentral),
            "CargandoEnCentral" => Ok(EstadoDron::CargandoEnCentral),
            _ => Err(DeserializationError::InvalidData),
        }
    }
}
use std::{collections::HashSet, io};
use std::str::FromStr;

use crate::{
    coordenadas::Coordenadas,
    csv::{csv_encodear_linea, csv_parsear_linea},
    serializables::{error::DeserializationError, Serializable, guardar::cargar_serializable}, 
    configuracion::Configuracion,
};

#[derive(Debug, Clone)]
pub enum EstadoDron {
    EnEspera,
    VolviendoACentroDeOperacion,
    YendoAIncidente,
    AtendiendoIncidente,
    YendoACentral,
    CargandoEnCentral,
    Error,
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
    pub duracion_bateria: u64, // En segundos
    pub bateria_minima: u64,
    pub incidentes_cercanos: HashSet<u64>,
    latitud_central: f64,
    longitud_central: f64,
    latitud_centro_operaciones: f64,
    longitud_centro_operaciones: f64,
    pub configuracion: Configuracion,
}

impl Dron {
    pub fn new(configuracion: Configuracion) -> Self {
        Dron {
            id: 0,
            latitud: 0.0,
            longitud: 0.0,
            rango: 0.0,
            estado: EstadoDron::EnEspera,
            direccion: 0.0,
            velocidad: 0.0,
            duracion_bateria: 0,
            bateria_minima: 0,
            incidentes_cercanos: HashSet::new(),
            latitud_central: 0.0,
            longitud_central: 0.0,
            latitud_centro_operaciones: 0.0,
            longitud_centro_operaciones: 0.0,
            configuracion,
        }
    }

    pub fn iniciar(&mut self) -> io::Result<()> {
        self.cargar_dron()?;

        Ok(())
    }

    fn cargar_dron(&mut self) -> io::Result<()> {
        let ruta_archivo_dron = self
            .configuracion
            .obtener::<String>("drones")
            .unwrap_or("dron.csv".to_string());

        let existe = std::path::Path::new(&ruta_archivo_dron).exists();

        if !existe {
            std::fs::File::create(&ruta_archivo_dron)?;
        }

        let dron: Dron = cargar_serializable(&ruta_archivo_dron)?;
        println!("\nDron: {:?}", dron);

        /*
        self.estado.incidentes.clear();
        self.estado.iniciar_dron(dron.clone());

        let (tx, rx) = mpsc::channel::<u64>();
        self.estado.descargar_bateria_dron(dron.clone(), tx);

        self.estado.dron = Some(dron.clone());

        println!("\nIncidentes en rango: {:?}", dron.incidentes_cercanos);
        */

        Ok(())
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

        let bateria_minima = parametros
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
            bateria_minima,
            incidentes_cercanos,
            latitud_central,
            longitud_central,
            latitud_centro_operaciones,
            longitud_centro_operaciones,
            configuracion: Configuracion::new(),
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

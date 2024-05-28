use std::collections::HashSet;
use std::time::Duration;

use crate::{
    serializables::Serializable,
    csv::csv_encodear_linea,
};

#[derive(Debug)]
pub enum EstadoDron {
    EnEspera,
    VolviendoACentroDeOperacion,
    YendoAIncidente,
    AtendiendoIncidente,
    YendoACentral,
    CargandoEnCentral,
}

pub struct Dron {
    pub id: u64,
    pub latitud: f64,
    pub longitud: f64,
    pub rango: f64,
    estado: EstadoDron,
    direccion: f64, // En grados, sentido horario, empezando desde el norte
    velocidad: f64,
    duracion_bateria: u64, // En segundos
    incidentes_cercanos: HashSet<u64>,
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
}

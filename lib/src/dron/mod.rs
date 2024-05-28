use std::collections::HashSet;

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
}

fn serializar_vector_incidentes(incidentes: &HashSet<u64>) -> String {
    incidentes
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(";")
}
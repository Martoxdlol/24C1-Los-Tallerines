use crate::{
    hilo::{id::IdHilo, instruccion::Instruccion},
    publicacion::Publicacion,
    suscripciones::{id::IdSuscripcion, suscripcion::Suscripcion},
};

use super::id::IdConexion;

#[derive(Debug)]
pub struct TickContexto {
    // pub suscripciones: Vec<Suscripcion>,
    // pub desuscripciones: Vec<IdSuscripcion>,
    // pub publicaciones: Vec<Publicacion>,
    pub instrucciones: Vec<Instruccion>,
    pub id_hilo: IdHilo,
    pub id_conexion: IdConexion,
}

impl TickContexto {
    pub fn new(id_hilo: IdHilo, id_conexion: IdConexion) -> Self {
        Self {
            instrucciones: Vec::new(),
            id_hilo,
            id_conexion,
        }
    }

    pub fn suscribir(&mut self, suscripcion: Suscripcion) {
        self.instrucciones.push(Instruccion::Suscribir(suscripcion))
    }

    pub fn desuscribir(&mut self, id_suscripcion: IdSuscripcion) {
        self.instrucciones
            .push(Instruccion::Desuscribir(self.id_conexion, id_suscripcion))
    }

    pub fn publicar(&mut self, publicacion: Publicacion) {
        self.instrucciones
            .push(Instruccion::NuevaPublicacion(publicacion))
    }

    pub fn suscripciones(&self) -> Vec<Suscripcion> {
        self.instrucciones
            .iter()
            .filter_map(|instruccion| match instruccion {
                Instruccion::Suscribir(suscripcion) => Some(suscripcion.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn desuscripciones(&self) -> Vec<IdSuscripcion> {
        self.instrucciones
            .iter()
            .filter_map(|instruccion| match instruccion {
                Instruccion::Desuscribir(_, id_suscripcion) => Some(id_suscripcion.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn publicaciones(&self) -> Vec<Publicacion> {
        self.instrucciones
            .iter()
            .filter_map(|instruccion| match instruccion {
                Instruccion::NuevaPublicacion(publicacion) => Some(publicacion.clone()),
                _ => None,
            })
            .collect()
    }
}

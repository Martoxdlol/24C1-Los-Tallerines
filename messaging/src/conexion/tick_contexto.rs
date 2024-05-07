use crate::{hilo::id::IdHilo, publicacion::Publicacion, suscripciones::{id::IdSuscripcion, suscripcion::Suscripcion}};

use super::id::IdConexion;

pub struct TickContexto {
    pub suscripciones: Vec<Suscripcion>,
    pub desuscripciones: Vec<(IdConexion, IdSuscripcion)>,
    pub publicaciones: Vec<Publicacion>,
    pub id_hilo: IdHilo
}

impl TickContexto {
    pub fn new(id_hilo: IdHilo) -> Self {
        Self {
            suscripciones: Vec::new(),
            desuscripciones: Vec::new(),
            publicaciones: Vec::new(),
            id_hilo
        }
    }

    pub fn suscribir(&mut self, suscripcion: Suscripcion) {
        self.suscripciones.push(suscripcion);
    }

    pub fn desuscribir(&mut self, desuscripcion: (IdConexion, IdSuscripcion)) {
        self.desuscripciones.push(desuscripcion);
    }

    pub fn publicar(&mut self, publicacion: Publicacion) {
        self.publicaciones.push(publicacion);
    }
}

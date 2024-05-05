use std::sync::mpsc::{Receiver, Sender};

use crate::{instruccion::Instruccion, publicacion::Publicacion};

/// Estructura de una suscripcion (Sub), con el canal de instrucciones, el
/// canal de publicaciones que tiene la punta receptora de un canal de 
/// publicaciones y el id de la suscripcion
pub struct Suscripcion {
    canal_instrucciones: Sender<Instruccion>,
    canal_publicaciones: Receiver<Publicacion>,
    id: String,
}

impl Suscripcion {
    pub fn new(
        canal_instrucciones: Sender<Instruccion>,
        canal_publicaciones: Receiver<Publicacion>,
        id: String,
    ) -> Self {
        Self {
            canal_instrucciones,
            canal_publicaciones,
            id,
        }
    }

    pub fn leer(&self) -> Option<Publicacion> {
        if let Ok(publicacion) = self.canal_publicaciones.try_recv() {
            Some(publicacion)
        } else {
            None
        }
    }
}

impl Drop for Suscripcion {
    fn drop(&mut self) {
        // Envio el mensaje de desuscribir al canal de instrucciones
        self.canal_instrucciones.send(Instruccion::Desuscribir {
            id_suscripcion: self.id.clone(),
        });
    }
}

impl Iterator for Suscripcion {
    type Item = Publicacion;

    fn next(&mut self) -> Option<Self::Item> {
        self.leer()
    }
}

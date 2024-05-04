use std::sync::mpsc::{Receiver, Sender};

use crate::{instruccion::Instruccion, publicacion::Publicacion};

pub struct Subscripcion {
    canal_instrucciones: Sender<Instruccion>,
    canal_publicaciones: Receiver<Publicacion>,
    id: String,
}

impl Subscripcion {
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

impl Drop for Subscripcion {
    fn drop(&mut self) {
        let _ = self.canal_instrucciones.send(Instruccion::Desubscribir {
            id_subscripcion: self.id.clone(),
        });
    }
}

impl Iterator for Subscripcion {
    type Item = Publicacion;

    fn next(&mut self) -> Option<Self::Item> {
        self.leer()
    }
}

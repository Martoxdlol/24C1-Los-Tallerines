use std::{
    sync::mpsc::{Receiver, RecvError, RecvTimeoutError, Sender, TryRecvError},
    time::Duration,
};

use super::{instruccion::Instruccion, publicacion::Publicacion};

/// Estructura de una suscripcion (Sub), con el canal de instrucciones, el
/// canal de publicaciones que tiene la punta receptora de un canal de
/// publicaciones y el id de la suscripcion
pub struct Suscripcion {
    canal_instrucciones: Sender<Instruccion>,
    canal_publicaciones: Receiver<Publicacion>,
    conectado: bool,
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
            conectado: true,
            id,
        }
    }

    pub fn leer(&self) -> Result<Publicacion, RecvError> {
        self.canal_publicaciones.recv()
    }

    pub fn intentar_leer(&mut self) -> Result<Option<Publicacion>, TryRecvError> {
        match self.canal_publicaciones.try_recv() {
            Ok(publicacion) => Ok(Some(publicacion)),
            Err(e) => {
                if let TryRecvError::Empty = e {
                    Ok(None)
                } else {
                    self.conectado = false;
                    Err(e)
                }
            }
        }
    }

    pub fn leer_con_limite_de_tiempo(
        &mut self,
        limite: Duration,
    ) -> Result<Publicacion, RecvTimeoutError> {
        match self.canal_publicaciones.recv_timeout(limite) {
            Ok(publicacion) => Ok(publicacion),
            Err(e) => {
                if let RecvTimeoutError::Timeout = e {
                    Err(e)
                } else {
                    self.conectado = false;
                    Err(e)
                }
            }
        }
    }
}

impl Drop for Suscripcion {
    fn drop(&mut self) {
        // Envio el mensaje de desuscribir al canal de instrucciones
        let _ = self.canal_instrucciones.send(Instruccion::Desuscribir {
            id_suscripcion: self.id.clone(),
        });
    }
}

impl Iterator for Suscripcion {
    type Item = Result<Publicacion, RecvError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.leer())
    }
}

use std::sync::mpsc::{channel, Sender};

use self::{hilo::hilo_registrador, registro::Registro};

mod hilo;
pub mod registro;

pub struct Registrador {
    emisor: Sender<Registro>
}

impl Registrador {
    pub fn new() -> Self {
        // crear channel
        let (emisor, receptor) = channel();
        hilo_registrador(receptor);

        Registrador { emisor }
    }

    pub fn info(&self, mensaje: String, hilo: Option<u64>, conexion: Option<u64>) {
        let _ = self.emisor.send(Registro::info(mensaje, hilo, conexion));
    }

    pub fn advertencia(&self, mensaje: String, hilo: Option<u64>, conexion: Option<u64>) {
        let _ = self.emisor.send(Registro::advertencia(mensaje, hilo, conexion));
    }

    pub fn error(&self, mensaje: String, hilo: Option<u64>, conexion: Option<u64>) {
        let _ = self.emisor.send(Registro::error(mensaje, hilo, conexion));
    }
}
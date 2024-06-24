use std::io;

use crate::cliente::{publicacion::Publicacion, suscripcion::Suscripcion};

use super::JetStream;

pub struct JSSuscripcion {
    pub suscripcion_actual: Suscripcion,
    pub stream_nombre: String,
    pub consumer_nombre: String,
    pub js: JetStream,
}

impl JSSuscripcion {
    pub fn new(
        js: JetStream,
        stream_nombre: String,
        consumer_nombre: String,
        suscripcion_inicial: Suscripcion,
    ) -> Self {
        Self {
            js,
            suscripcion_actual: suscripcion_inicial,
            stream_nombre,
            consumer_nombre,
        }
    }

    pub fn intentar_siguiente_mensaje(&mut self) -> io::Result<Option<Publicacion>> {
        match self.suscripcion_actual.intentar_leer() {
            Ok(Some(publicacion)) => {
                self.suscripcion_actual = self
                    .js
                    .suscribir_proximo_mensaje(&self.stream_nombre, &self.consumer_nombre)?;

                Ok(Some(publicacion))
            }
            _ => Ok(None),
        }
    }
}

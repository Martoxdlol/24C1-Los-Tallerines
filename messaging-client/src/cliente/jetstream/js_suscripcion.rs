use std::io;

use crate::cliente::{publicacion::Publicacion, suscripcion::Suscripcion};

use super::JetStream;

pub struct JSSuscripcion {
    pub suscripcion_actual: Suscripcion,
    pub sub_utilizada: bool,
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
            sub_utilizada: false,
        }
    }

    pub fn intentar_leer(&mut self) -> io::Result<Option<Publicacion>> {
        if self.sub_utilizada {
            self.sub_utilizada = false;
            self.suscripcion_actual = self
                .js
                .suscribir_proximo_mensaje(&self.stream_nombre, &self.consumer_nombre)?;
        }

        match self.suscripcion_actual.intentar_leer() {
            Ok(Some(publicacion)) => {
                self.sub_utilizada = true;

                Ok(Some(publicacion))
            }
            _ => Ok(None),
        }
    }

    pub fn ack(&mut self, publicacion: &Publicacion) -> io::Result<()> {
        self.js.ack(publicacion)
    }
}

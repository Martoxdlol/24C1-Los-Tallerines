use std::io;

use crate::cliente::{publicacion::Publicacion, suscripcion::Suscripcion};

use super::{constantes::js_api_consumer_next, JetStream};

pub struct JSSuscripcion {
    pub stream_nombre: String,
    pub consumer_nombre: String,
    pub js: JetStream,
    pub ack_pendiente: Option<String>,
    pub suscripcion: Option<Suscripcion>,
    pub inbox: String,
}

impl JSSuscripcion {
    pub fn new(js: JetStream, stream_nombre: String, consumer_nombre: String) -> Self {
        Self {
            js,
            stream_nombre,
            consumer_nombre,
            ack_pendiente: None,
            suscripcion: None,
            inbox: nuid::next().to_string(),
        }
    }

    pub fn suscribirse_next(&mut self) -> io::Result<()> {
        let topico = js_api_consumer_next(&self.stream_nombre, &self.consumer_nombre);

        self.js.cliente.publicar(&topico, b"1", Some(&self.inbox))?;

        let sub = self.js.cliente.suscribirse(&self.inbox, None)?;

        self.suscripcion = Some(sub);

        Ok(())
    }

    pub fn intentar_leer(&mut self) -> io::Result<Option<Publicacion>> {
        self.ack()?;

        if self.suscripcion.is_none() {
            self.suscribirse_next()?;
        }

        if let Some(sub) = &self.suscripcion {
            if let Some(publicacion) = sub.intentar_leer()? {
                self.ack_pendiente.clone_from(&publicacion.reply_to);
                self.suscripcion = None;
                return Ok(Some(publicacion));
            }
        }

        Ok(None)
    }

    pub fn ack(&mut self) -> io::Result<()> {
        if let Some(ack) = &self.ack_pendiente {
            self.js.cliente.publicar(ack, b"", None)?;
            self.ack_pendiente = None;
        }

        Ok(())
    }

    pub fn no_ack(&mut self) -> io::Result<()> {
        self.ack_pendiente = None;
        Ok(())
    }
}

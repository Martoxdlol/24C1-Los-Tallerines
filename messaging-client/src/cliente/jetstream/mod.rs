use std::{io, time::Duration};

use constantes::{js_api_consumer_next, js_api_stream_create};
use lib::jet_stream::{
    consumer_config::ConsumerConfig, crear_consumer_peticion::JSPeticionCrearConsumer,
    stream_config::StreamConfig,
};

use super::{publicacion::Publicacion, suscripcion::Suscripcion, Cliente};

pub mod constantes;

pub struct JetStream {
    pub cliente: Cliente,
}

impl JetStream {
    pub fn new(cliente: Cliente) -> Self {
        Self { cliente }
    }

    pub fn publicar(&self, subject: &str, data: &[u8]) -> io::Result<()> {
        self.cliente.publicar(subject, data, None)
    }

    pub fn crear_stream(&mut self, config: &StreamConfig) -> io::Result<()> {
        let body = config
            .to_json()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        self.cliente.peticion_tiempo_limite(
            &js_api_stream_create(&config.name),
            body.as_bytes(),
            Duration::from_secs(5),
        )?;
        Ok(())
    }

    pub fn crear_consumer(&mut self, config: ConsumerConfig) -> io::Result<()> {
        let peticion = JSPeticionCrearConsumer::new(config);
        let body = peticion
            .to_json()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        self.cliente.peticion_tiempo_limite(
            &js_api_stream_create(&peticion.config.durable_name),
            body.as_bytes(),
            Duration::from_secs(5),
        )?;
        Ok(())
    }

    pub fn suscribir_proximo_mensaje(
        &mut self,
        stream_name: &str,
        consumer_name: &str,
    ) -> io::Result<Suscripcion> {
        let topico = js_api_consumer_next(stream_name, consumer_name);

        let inbox = self.cliente.nuevo_inbox();

        let sub = self.cliente.suscribirse(&inbox, None)?;

        self.cliente.publicar(&topico, b"1", Some(&inbox))?;

        Ok(sub)
    }

    pub fn ack(&self, publicacion: &Publicacion) -> io::Result<()> {
        if let Some(reply_to) = &publicacion.reply_to {
            self.cliente.publicar(reply_to, b"", None)?;
        }

        Ok(())
    }
}

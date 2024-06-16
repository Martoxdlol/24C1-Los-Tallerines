use std::sync::mpsc::Sender;

use chrono::Utc;
use lib::jet_stream::{
    consumer_config::ConsumerConfig, consumer_info::ConsumerInfo,
    consumer_info_respuesta::JSConsumerInfoRespuesta,
};

use crate::{
    conexion::{r#trait::Conexion, tick_contexto::TickContexto},
    publicacion::Publicacion,
    suscripciones::{suscripcion::Suscripcion, topico::Topico},
};

use super::actualizacion::ActualizacionJS;

pub struct JetStreamConsumer {
    id_conexion: u64,
    nombre_stream: String,
    config: ConsumerConfig,
    eliminado: bool,
    preparado: bool,
    tx_actualizaciones_js: Sender<ActualizacionJS>,
    respuestas: Vec<Publicacion>,
}

impl JetStreamConsumer {
    pub fn new(
        config: ConsumerConfig,
        nombre_stream: String,
        tx_actualizaciones_js: Sender<ActualizacionJS>,
    ) -> Self {
        JetStreamConsumer {
            nombre_stream,
            id_conexion: 0,
            config,
            eliminado: false,
            preparado: false,
            tx_actualizaciones_js,
            respuestas: Vec::new(),
        }
    }

    fn suscribir(&self, contexto: &mut TickContexto, topico: &str, sid: &str) {
        contexto.suscribir(Suscripcion::new(
            contexto.id_hilo,
            self.id_conexion,
            Topico::new(topico.to_string()).unwrap(),
            sid.to_string(),
            None,
        ));
    }

    fn enviar_actualizacion_de_estado(&self) {
        let _ = self
            .tx_actualizaciones_js
            .send(ActualizacionJS::Consumer(ConsumerInfo {
                config: self.config.clone(),
                created: Utc::now().to_rfc3339(),
                ts: Utc::now().to_rfc3339(),
            }));
    }
}

impl Conexion for JetStreamConsumer {
    fn obtener_id(&self) -> u64 {
        self.id_conexion
    }

    fn setear_id_conexion(&mut self, id_conexion: u64) {
        self.id_conexion = id_conexion;
    }

    fn tick(&mut self, contexto: &mut crate::conexion::tick_contexto::TickContexto) {
        if !self.preparado {
            self.suscribir(
                contexto,
                &format!(
                    "$JS.API.CONSUMER.INFO.{}.{}",
                    self.nombre_stream, self.config.durable_name
                ),
                "info",
            );
            self.suscribir(
                contexto,
                &format!(
                    "$JS.API.CONSUMER.DELETE.{}.{}",
                    self.nombre_stream, self.config.durable_name
                ),
                "eliminar",
            );
            self.suscribir(
                contexto,
                &format!(
                    "$JS.API.CONSUMER.MSG.NEXT.{}.{}",
                    self.nombre_stream, self.config.durable_name
                ),
                "mensaje_siguiente",
            );

            self.enviar_actualizacion_de_estado();

            self.preparado = true;
        }

        for respuesta in self.respuestas.drain(..) {
            contexto.publicar(respuesta);
        }
    }

    fn escribir_publicacion_mensaje(
        &mut self,
        mensaje: &crate::publicacion::mensaje::PublicacionMensaje,
    ) {
        match mensaje.sid.as_str() {
            "info" => {
                if let Some(reply_to) = &mensaje.replay_to {
                    if let Ok(respuesta) =
                        JSConsumerInfoRespuesta::new(self.config.clone()).to_json()
                    {
                        self.respuestas.push(Publicacion::new(
                            reply_to.to_string(),
                            respuesta.as_bytes().to_owned(),
                            None,
                            None,
                        ));
                    }
                }
            }
            "eliminar" => {
                self.eliminado = true;
                let _ = self
                    .tx_actualizaciones_js
                    .send(ActualizacionJS::ConsumerEliminado(
                        self.config.durable_name.clone(),
                    ));
            }
            _ => {}
        }
    }

    fn esta_conectado(&self) -> bool {
        !self.eliminado
    }
}

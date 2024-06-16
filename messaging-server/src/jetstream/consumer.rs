use std::sync::mpsc::{Receiver, Sender};

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
    mensaje_pendiente: Option<Publicacion>,
    rx_mensajes: Receiver<Publicacion>,
    topico_ack_mensaje_pendiente: String,
}

impl JetStreamConsumer {
    pub fn new(
        config: ConsumerConfig,
        nombre_stream: String,
        tx_actualizaciones_js: Sender<ActualizacionJS>,
        rx_mensajes: Receiver<Publicacion>,
    ) -> Self {
        JetStreamConsumer {
            nombre_stream,
            id_conexion: 0,
            config,
            eliminado: false,
            preparado: false,
            tx_actualizaciones_js,
            respuestas: Vec::new(),
            mensaje_pendiente: None,
            rx_mensajes,
            topico_ack_mensaje_pendiente: "".to_string(),
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

    fn responder_mensaje_pendiente(&mut self) {
        let ack_topico = format!(
            "$JS.ACK.{}.{}.{}",
            self.nombre_stream,
            self.config.durable_name,
            nuid::next()
        );

        if let Some(mensaje) = &self.mensaje_pendiente {
            self.topico_ack_mensaje_pendiente = ack_topico.clone();
            self.respuestas.push(Publicacion::new(
                mensaje.topico.clone(),
                mensaje.payload.clone(),
                mensaje.header.clone(),
                Some(ack_topico),
            ));
        }
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
            self.suscribir(
                contexto,
                &format!(
                    "$JS.ACK.{}.{}.*",
                    self.nombre_stream, self.config.durable_name
                ),
                "ack",
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
            "mensaje_siguiente" => {
                if self.mensaje_pendiente.is_none() {
                    if let Ok(mensaje) = self.rx_mensajes.try_recv() {
                        self.mensaje_pendiente = Some(mensaje);
                    }
                }

                self.responder_mensaje_pendiente();
            }
            "ack" => {
                if let Some(mensaje) = &self.mensaje_pendiente {
                    if self.topico_ack_mensaje_pendiente.eq(&mensaje.topico) {
                        self.mensaje_pendiente = None;
                        self.topico_ack_mensaje_pendiente = "".to_string();
                    }
                }
            }
            _ => {}
        }
    }

    fn esta_conectado(&self) -> bool {
        !self.eliminado
    }
}

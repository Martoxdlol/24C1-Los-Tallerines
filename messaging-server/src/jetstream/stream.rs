use std::sync::mpsc::Sender;

use chrono::Utc;
use lib::jet_stream::{
    stream_config::StreamConfig, stream_info::StreamInfo,
    stream_info_respuesta::JSStreamInfoRespuesta, stream_state::JetStreamStreamState,
};

use crate::{
    conexion::{r#trait::Conexion, tick_contexto::TickContexto},
    publicacion::Publicacion,
    suscripciones::{suscripcion::Suscripcion, topico::Topico},
};

use super::actualizacion::ActualizacionJS;

pub struct JetStreamStream {
    id_conexion: u64,
    config: StreamConfig,
    eliminado: bool,
    preparado: bool,
    tx_actualizaciones_js: Sender<ActualizacionJS>,
    respuestas: Vec<Publicacion>,
}

impl JetStreamStream {
    pub fn new(config: StreamConfig, tx_actualizaciones_js: Sender<ActualizacionJS>) -> Self {
        JetStreamStream {
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
            .send(ActualizacionJS::Stream(StreamInfo {
                config: self.config.clone(),
                created: Utc::now().to_rfc3339(),
                state: JetStreamStreamState::new(),
                ts: Utc::now().to_rfc3339(),
            }));
    }
}

impl Conexion for JetStreamStream {
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
                &format!("$JS.API.STREAM.INFO.{}", self.config.name),
                "info",
            );
            self.suscribir(
                contexto,
                &format!("$JS.API.STREAM.DELETE.{}", self.config.name),
                "eliminar",
            );
            self.suscribir(
                contexto,
                &format!("$JS.API.STREAM.UPDATE.{}", self.config.name),
                "actualizar",
            );
            self.suscribir(
                contexto,
                &format!("$JS.API.STREAM.PURGE.{}", self.config.name),
                "purgar",
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
                        JSStreamInfoRespuesta::new(self.config.clone(), JetStreamStreamState::new())
                            .to_json()
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
                    .send(ActualizacionJS::StreamEliminado(self.config.name.clone()));
            }
            "actualizar" => {}
            "purgar" => {}
            _ => {}
        }
    }

    fn esta_conectado(&self) -> bool {
        !self.eliminado
    }
}

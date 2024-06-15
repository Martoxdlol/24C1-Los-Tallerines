use std::sync::mpsc::Sender;

use lib::jet_stream::stream_config::StreamConfig;

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
            "info" => {}
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

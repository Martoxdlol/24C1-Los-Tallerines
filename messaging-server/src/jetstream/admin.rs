use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender},
};

use lib::jet_stream::{stream_config::StreamConfig, stream_info::StreamInfo};

use crate::{
    conexion::{r#trait::Conexion, tick_contexto::TickContexto},
    publicacion::Publicacion,
    suscripciones::{suscripcion::Suscripcion, topico::Topico},
};

use super::{actualizacion::ActualizacionJS, stream::JetStreamStream};

pub struct JestStreamAdminConexion {
    id: u64,
    preparado: bool,
    tx_conexiones: Sender<Box<dyn Conexion + Send>>,
    respuestas: Vec<Publicacion>,
    streams: HashMap<String, StreamInfo>,
    rx_datos_js: Receiver<ActualizacionJS>,
    tx_datos_js: Sender<ActualizacionJS>,
}

impl JestStreamAdminConexion {
    pub fn new(
        id: u64,
        tx_conexiones: Sender<Box<dyn Conexion + Send>>,
    ) -> JestStreamAdminConexion {
        let (tx_datos_js, rx_datos_js) = channel();

        JestStreamAdminConexion {
            preparado: false,
            id,
            tx_conexiones,
            respuestas: Vec::new(),
            streams: HashMap::new(),
            rx_datos_js,
            tx_datos_js,
        }
    }

    fn suscribir(&self, contexto: &mut TickContexto, topico: &str, sid: &str) {
        contexto.suscribir(Suscripcion::new(
            contexto.id_hilo,
            self.id,
            Topico::new(topico.to_string()).unwrap(),
            sid.to_string(),
            None,
        ));
    }

    fn recibir_actualizaciones_js(&mut self) {
        while let Ok(actualizacion) = self.rx_datos_js.try_recv() {
            match actualizacion {
                ActualizacionJS::Stream(stream_info) => {
                    self.streams
                        .insert(stream_info.config.name.clone(), stream_info);
                }
                ActualizacionJS::StreamEliminado(nombre) => {
                    self.streams.remove(&nombre);
                }
            }
        }
    }

    fn crear_stream(&self, config: StreamConfig) {
        let stream = JetStreamStream::new(config, self.tx_datos_js.clone());
        let _ = self.tx_conexiones.send(Box::new(stream));
    }
}

impl Conexion for JestStreamAdminConexion {
    fn obtener_id(&self) -> u64 {
        self.id
    }

    fn setear_id_conexion(&mut self, id_conexion: u64) {
        self.id = id_conexion;
    }

    fn tick(&mut self, contexto: &mut TickContexto) {
        if !self.preparado {
            self.suscribir(contexto, "$JS.API.STREAM.CREATE.*", "stream.crear");
            self.suscribir(contexto, "$JS.API.STREAM.LIST", "stream.listar");
            self.suscribir(contexto, "$JS.API.STREAM.NAMES", "stream.nombres");
            self.preparado = true;
        }

        for respuesta in self.respuestas.drain(..) {
            contexto.publicar(respuesta);
        }

        self.recibir_actualizaciones_js();
    }

    fn escribir_publicacion_mensaje(
        &mut self,
        mensaje: &crate::publicacion::mensaje::PublicacionMensaje,
    ) {
        match mensaje.sid.as_str() {
            "stream.crear" => {
                if let Ok(config) =
                    StreamConfig::from_json(&String::from_utf8_lossy(&mensaje.payload))
                {
                    self.crear_stream(config);
                }
            }
            "stream.listar" => {
                println!("JestStreamHilo::escribir_publicacion_mensaje: stream.listar");
            }
            "stream.nombres" => {
                println!("JestStreamHilo::escribir_publicacion_mensaje: stream.nombres");
            }
            _ => {}
        }
    }

    fn esta_conectado(&self) -> bool {
        true
    }
}

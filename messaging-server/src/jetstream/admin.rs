use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender},
};

use lib::jet_stream::{
    admin_nombres_streams_respuesta::JSNombresStreamsRespuesta,
    api_info_response::JSApiInfoResponse, crear_stream_respuesta::JSCrearStreamRespuesta,
    stream_config::StreamConfig, stream_info::StreamInfo,
    stream_list_response::JetStreamStreamListResponse,
};

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
            println!(
                "JestStreamAdminConexion::recibir_actualizaciones_js: {:?}",
                actualizacion
            );
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
            self.suscribir(contexto, "$JS.API.INFO", "info");
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
            "info" => {
                // Si hay reply_to, es una respuesta a una petición
                if let Some(reply_to) = &mensaje.replay_to {
                    if let Ok(info) =
                        JSApiInfoResponse::new(self.streams.values().len() as i32, 0).to_json()
                    {
                        self.respuestas.push(Publicacion::new(
                            reply_to.to_string(),
                            info.as_bytes().to_owned(),
                            None,
                            None,
                        ))
                    }
                }
            }
            "stream.crear" => {
                if let Ok(config) =
                    StreamConfig::from_json(&String::from_utf8_lossy(&mensaje.payload))
                {
                    self.crear_stream(config.clone());

                    if let Some(reply_to) = &mensaje.replay_to {
                        if let Ok(respuesta) = JSCrearStreamRespuesta::new(config, true).to_json() {
                            self.respuestas.push(Publicacion::new(
                                reply_to.to_string(),
                                respuesta.as_bytes().to_owned(),
                                None,
                                None,
                            ));
                        }
                    }
                }
            }
            "stream.listar" => {
                println!("JestStreamHilo::escribir_publicacion_mensaje: stream.listar");
                if let Some(reply_to) = &mensaje.replay_to {
                    let streams_info = self
                        .streams
                        .values()
                        .map(|s| s.clone())
                        .collect::<Vec<StreamInfo>>();

                    let r = JetStreamStreamListResponse {
                        limit: (streams_info.len() + 1) as i32,
                        total: streams_info.len() as i32,
                        streams: streams_info,
                        r#type: "io.nats.jetstream.api.v1.stream_list_response".to_string(),
                    };

                    if let Ok(respuesta) = r.to_json() {
                        self.respuestas.push(Publicacion::new(
                            reply_to.to_string(),
                            respuesta.as_bytes().to_owned(),
                            None,
                            None,
                        ));
                    }
                }
            }
            "stream.nombres" => {
                let nombres_streams = self
                    .streams
                    .keys()
                    .map(|k| k.to_string())
                    .collect::<Vec<String>>();
                if let Some(reply_to) = &mensaje.replay_to {
                    if let Ok(respuesta) = JSNombresStreamsRespuesta::new(nombres_streams).to_json()
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
            _ => {}
        }
    }

    fn esta_conectado(&self) -> bool {
        true
    }
}

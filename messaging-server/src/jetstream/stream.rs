use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender},
};

use chrono::Utc;
use lib::jet_stream::{
    consumer_config::ConsumerConfig, consumer_info::ConsumerInfo,
    consumer_list_respuesta::JetStreamConsumerListaRespuesta,
    crear_consumer_peticion::JSPeticionCrearConsumer,
    crear_consumer_respuesta::JSCrearConsumerRespuesta,
    nombres_consumers_respuesta::JSNombresConsumersRespuesta, stream_config::StreamConfig,
    stream_info::StreamInfo, stream_info_respuesta::JSStreamInfoRespuesta,
    stream_state::JetStreamStreamState,
};

use crate::{
    conexion::{r#trait::Conexion, tick_contexto::TickContexto},
    publicacion::Publicacion,
    registrador::Registrador,
    suscripciones::{suscripcion::Suscripcion, topico::Topico},
};

use super::{actualizacion::ActualizacionJS, consumer::JetStreamConsumer};

pub struct JetStreamStream {
    id_conexion: u64,
    config: StreamConfig,
    eliminado: bool,
    preparado: bool,
    tx_conexiones: Sender<Box<dyn Conexion + Send>>,
    tx_actualizaciones_js: Sender<ActualizacionJS>,
    rx_actualizaciones_js_consumers: Receiver<ActualizacionJS>,
    tx_actualizaciones_js_consumers: Sender<ActualizacionJS>,
    respuestas: Vec<Publicacion>,
    consumers: HashMap<String, ConsumerInfo>,
    consumers_transmisores: HashMap<String, Sender<Publicacion>>,
    registrador: Registrador,
}

impl JetStreamStream {
    pub fn new(
        config: StreamConfig,
        tx_actualizaciones_js: Sender<ActualizacionJS>,
        tx_conexiones: Sender<Box<dyn Conexion + Send>>,
        registrador: Registrador,
    ) -> Self {
        let (tx_actualizaciones_js_consumers, rx_actualizaciones_js_consumers) = channel();

        JetStreamStream {
            tx_conexiones,
            id_conexion: 0,
            config,
            eliminado: false,
            preparado: false,
            tx_actualizaciones_js,
            respuestas: Vec::new(),
            rx_actualizaciones_js_consumers,
            tx_actualizaciones_js_consumers,
            consumers: HashMap::new(),
            consumers_transmisores: HashMap::new(),
            registrador,
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

    fn recibir_actualizaciones_js_consumers(&mut self) {
        while let Ok(actualizacion) = self.rx_actualizaciones_js_consumers.try_recv() {
            match actualizacion {
                ActualizacionJS::Consumer(consumer_info) => {
                    self.consumers
                        .insert(consumer_info.config.durable_name.clone(), consumer_info);
                }
                ActualizacionJS::ConsumerEliminado(durable_name) => {
                    self.consumers.remove(&durable_name);
                    self.consumers_transmisores.remove(&durable_name);
                }
                _ => {}
            }
        }
    }

    fn crear_consumer(&mut self, config: ConsumerConfig) {
        let (tx, rx) = channel();

        self.consumers_transmisores
            .insert(config.durable_name.clone(), tx);

        self.registrador.info(
            &format!("Consumer creado: {:?}", config),
            Some(self.obtener_id()),
        );

        let consumer = JetStreamConsumer::new(
            config,
            self.config.name.clone(),
            self.tx_actualizaciones_js_consumers.clone(),
            rx,
            self.registrador.clone(),
        );

        let _ = self.tx_conexiones.send(Box::new(consumer));
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
            self.suscribir(
                contexto,
                &format!("$JS.API.CONSUMER.CREATE.{}.>", self.config.name),
                "crear_consumer",
            );
            self.suscribir(
                contexto,
                &format!("$JS.API.CONSUMER.LIST.{}", self.config.name),
                "listar_consumers",
            );
            self.suscribir(
                contexto,
                &format!("$JS.API.CONSUMER.NAMES.{}", self.config.name),
                "nombres_consumer",
            );

            for topico in self.config.subjects.iter() {
                self.suscribir(contexto, topico, &format!("mensaje|{}", topico));
            }

            self.enviar_actualizacion_de_estado();

            self.preparado = true;
        }

        self.recibir_actualizaciones_js_consumers();

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
            "crear_consumer" => {
                if let Ok(datos) =
                    JSPeticionCrearConsumer::from_json(&String::from_utf8_lossy(&mensaje.payload))
                {
                    let mut creado = true;
                    if self.consumers.contains_key(&datos.config.durable_name) {
                        creado = false;
                    } else {
                        self.crear_consumer(datos.config.clone());
                    }

                    if let Some(reply_to) = &mensaje.replay_to {
                        if let Ok(respuesta) =
                            JSCrearConsumerRespuesta::new(datos.config, creado).to_json()
                        {
                            self.respuestas.push(Publicacion::new(
                                reply_to.to_string(),
                                respuesta.as_bytes().to_owned(),
                                None,
                                None,
                            ));
                        } else {
                            self.registrador.error(
                                "Error al serializar respuesta de crear consumer",
                                Some(self.obtener_id()),
                            );
                        }
                    }
                }
            }
            "listar_consumers" => {
                if let Some(reply_to) = &mensaje.replay_to {
                    let consumers_info = self
                        .consumers
                        .values()
                        .cloned()
                        .collect::<Vec<ConsumerInfo>>();

                    let r = JetStreamConsumerListaRespuesta {
                        limit: (consumers_info.len() + 1) as i32,
                        total: consumers_info.len() as i32,
                        consumers: consumers_info,
                        r#type: "io.nats.jetstream.api.v1.consumer_list_response".to_string(),
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
            "nombres_consumer" => {
                let nombres_consumers = self
                    .consumers
                    .keys()
                    .map(|k| k.to_string())
                    .collect::<Vec<String>>();
                if let Some(reply_to) = &mensaje.replay_to {
                    if let Ok(respuesta) =
                        JSNombresConsumersRespuesta::new(nombres_consumers).to_json()
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

        if mensaje.sid.starts_with("mensaje|") {
            for (nombre_consumer, tx_consumer) in self.consumers_transmisores.iter() {
                if let Some(consumer) = self.consumers.get(nombre_consumer) {
                    if !consumer_aceptar_topico(&consumer.config, &mensaje.topico) {
                        continue;
                    }

                    if tx_consumer
                        .send(Publicacion::new(
                            mensaje.topico.clone(),
                            mensaje.payload.clone(),
                            mensaje.header.clone(),
                            mensaje.replay_to.clone(),
                        ))
                        .is_err()
                    {
                        self.registrador.error(
                            &format!("Error al enviar mensaje a consumer {}", nombre_consumer),
                            Some(self.obtener_id()),
                        );
                    }
                } else {
                    self.registrador.error(
                        &format!("Consumer {} no encontrado", nombre_consumer),
                        Some(self.obtener_id()),
                    );
                }
            }
        }
    }

    fn esta_conectado(&self) -> bool {
        !self.eliminado
    }
}

pub fn consumer_aceptar_topico(config: &ConsumerConfig, topico: &str) -> bool {
    if let Some(filter_subject) = &config.filter_subject {
        if let Ok(topico_consumer) = Topico::new(filter_subject.clone()) {
            return topico_consumer.test(topico);
        }
    } else if let Some(filter_subjects) = &config.filter_subjects {
        if filter_subjects.is_empty() {
            return true;
        }
        for filter_subject in filter_subjects {
            if let Ok(topico_consumer) = Topico::new(filter_subject.clone()) {
                if topico_consumer.test(topico) {
                    return true;
                }
            }
        }
    } else {
        return true;
    }

    false
}

pub mod id;
pub mod message;
pub mod respuesta;
pub mod tick_contexto;
use std::{fmt::Debug, io};

use crate::{
    parseador::Parseador, publicacion::Publicacion, publicacion_mensaje::PublicacionMensaje,
    registrador::Registrador, stream::Stream, suscripciones::suscripcion::Suscripcion,
    topico::Topico,
};

use self::{id::IdConexion, message::Message, respuesta::Respuesta, tick_contexto::TickContexto};
pub struct Conexion {
    /// El identificador de la conexión. Global y único
    id: IdConexion,
    /// El stream de la conexión
    stream: Box<dyn Stream>,
    /// Registrador de eventos
    registrador: Registrador,
    /// El parser se encarga de leer los bytes y generar mensajes
    parser: Parseador,

    pub desconectado: bool,

    /// Indica si la conexión está autenticada.
    /// Es decir, si ya se envió un mensaje de conexión (`CONNECT {...}`)
    pub autenticado: bool,
}

impl Conexion {
    pub fn new(id: IdConexion, stream: Box<dyn Stream>, registrador: Registrador) -> Self {
        let mut con = Self {
            id,
            stream,
            parser: Parseador::new(),
            registrador,
            desconectado: false,
            autenticado: false,
        };

        con.enviar_info();

        return con;
    }

    pub fn tick(&mut self, salida: &mut TickContexto) {
        if self.desconectado {
            return;
        }

        // Lee los bytes del stream y los envía al parser
        self.leer_bytes();

        // Lee mensaje y actua en consecuencia
        self.leer_mensajes(salida);
    }

    /// Este método lo envia el Hilo cuando recibe un mensaje
    pub fn escribir_publicacion_mensaje(&mut self, mensaje: &PublicacionMensaje) {
        self.registrador
            .info(&format!("MSG: {:?}", mensaje), Some(self.id));

        if let Err(_) = self.escribir_bytes(&mensaje.serializar_msg()) {
            self.registrador
                .advertencia("Error al enviar mensaje", Some(self.id));
        }
    }

    /// Lee los bytes del stream y los envía al parser
    pub fn leer_bytes(&mut self) {
        let mut buffer = [0; 1024]; // 1kb
                                    // 1. Leer una vez
        match self.stream.read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    self.desconectado = true;
                    return;
                }

                // 2. Enviar bytes a parser y leer nuevos mensajes generados
                self.parser.agregar_bytes(&buffer[..n]);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No hay datos para leer (no hay que hacer nada acá)
            }
            Err(e) => {
                self.registrador
                    .error(&format!("Error al leer del stream {}", e), Some(self.id));
                self.registrador.error("Error al leer bytes", Some(self.id));

                self.desconectado = true;
                return;
            }
        }
    }

    /// Escribir al stream
    pub fn escribir_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        if let Err(e) = self.stream.write_all(bytes) {
            self.registrador
                .advertencia(&format!("Error al escribir al stream {}", e), Some(self.id));
            self.desconectado = true;
            return Err(e);
        }

        Ok(())
    }

    pub fn escribir_respuesta(&mut self, respuesta: &Respuesta) {
        let bytes = &respuesta.serializar();
        if let Err(_) = self.escribir_bytes(&bytes) {
            self.registrador
                .error("Error al enviar respuesta", Some(self.id));
        }
    }

    pub fn escribir_ok(&mut self, msg: Option<String>) {
        self.escribir_respuesta(&Respuesta::Ok(msg));
    }

    pub fn escribir_err(&mut self, msg: Option<String>) {
        self.escribir_respuesta(&Respuesta::Err(msg));
    }

    pub fn enviar_info(&mut self) {
        self.escribir_respuesta(&Respuesta::Info());
    }

    pub fn leer_mensajes(&mut self, contexto: &mut TickContexto) {
        while let Some(mensaje) = self.parser.proximo_mensaje() {
            self.registrador
                .info(&format!("Mensaje recibido: {:?}", mensaje), Some(self.id));

            if !self.autenticado {
                match mensaje {
                    Message::Connect(_) => {
                        self.autenticado = true;
                        self.escribir_respuesta(&Respuesta::Ok(Some("connect".to_string())));
                    }
                    _ => {
                        self.escribir_err(Some(
                            "Primero debe enviar un mensaje de conexión".to_string(),
                        ));
                        self.desconectado = true;
                        return;
                    }
                }
                continue;
            }

            // proximo mensaje va a leer los bytes nuevos y devuelve si es una accion valida
            match mensaje {
                Message::Pub(subject, replay_to, payload) => {
                    self.registrador.info(
                        &format!("Publicación: {:?} {:?} {:?}", subject, replay_to, payload),
                        Some(self.id),
                    );

                    contexto.publicar(Publicacion::new(subject, payload, None, replay_to));
                    self.escribir_ok(Some("pub".to_string()));
                }
                Message::Hpub(subject, replay_to, headers, payload) => {
                    self.registrador.info(
                        &format!(
                            "Publicación con header: {:?} {:?} {:?} {:?}",
                            subject, headers, replay_to, payload
                        ),
                        Some(self.id),
                    );

                    contexto.publicar(Publicacion::new(subject, payload, Some(headers), replay_to));
                    self.escribir_ok(Some("hpub".to_string()));
                }
                Message::Sub(topico, grupo, id) => match Topico::new(topico) {
                    Ok(topico) => {
                        contexto.suscribir(Suscripcion::new(
                            contexto.id_hilo,
                            self.id,
                            topico,
                            id,
                            grupo,
                        ));
                        self.escribir_ok(Some("sub".to_string()));
                    }
                    Err(_) => {
                        self.escribir_err(Some("Tópico de subscripción incorrecto".to_string()));
                    }
                },
                Message::Unsub(id, _max_msgs) => {
                    contexto.desuscribir(id);
                    self.escribir_ok(Some("unsub".to_string()));
                }
                Message::Err(msg) => {
                    // self.respuestas.push(Respuesta::Err(msg));
                    self.escribir_err(Some(msg));
                }
                Message::Connect(_) => {
                    self.escribir_err(Some("Ya se recibió un mensaje de conexión".to_string()));
                }
                Message::Ping() => {
                    self.escribir_respuesta(&Respuesta::Pong());
                }
            }
        }
    }

    pub fn esta_conectado(&self) -> bool {
        !self.desconectado
    }
}

impl Debug for Conexion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Conexion")
            .field("id", &self.id)
            .field("desconectado", &self.desconectado)
            .field("autenticado", &self.autenticado)
            .finish()
    }
}

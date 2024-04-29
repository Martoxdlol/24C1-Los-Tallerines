use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::TcpStream,
};

use super::{
    message::Message, parser::Parser, publicacion::Publicacion, respuesta::Respuesta,
    subject::Subject,
}; // Usamos super porque estamos en el mismo módulo

/// La conexión es responsable de mantener el stream con el cliente, leer mensajes y enviar mensajes
///
/// El proceso este se va realizando en el tiempo a través de llamar el método `tick` en un loop
pub struct Conexion {
    /// El stream de la conexión
    stream: TcpStream,
    /// El parser se encarga de leer los bytes y generar mensajes
    parser: Parser,
    /// Por cada conexion, vamos a guardar los topicos a los que se suscribio
    subscripciones: HashMap<String, Subject>,
    /// Las publicaciones que manda el cliente
    publicaciones_salientes: Vec<Publicacion>,
    /// Las respuestas y publicaciones que se envian al stream del cliente
    respuestas: Vec<Respuesta>,
    /// Flag para saber si la conexión está activa
    recibi_connect: bool,
}

impl Conexion {
    pub fn new(stream: TcpStream) -> Self {
        let mut respuestas = Vec::new();
        respuestas.push(Respuesta::Info());
        Self {
            stream: stream, // Los bytes de donde vamos a saber: QUE hay que hacer en DONDE, y si es publicar, el mensaje
            parser: Parser::new(),
            subscripciones: HashMap::new(),
            publicaciones_salientes: Vec::new(),
            respuestas: respuestas,
            recibi_connect: false,
        }
    }

    /// Lee los mensajes nuevos recibidos del stream y que fueron previamente enviados al parser
    pub fn leer_mensajes(&mut self) {
        while let Some(mensaje) = self.parser.proximo_mensaje() {
            // Devuelve que tipo de mensaje es
            println!("Mensaje: {:?}", mensaje);

            if self.recibi_connect == false {
                match mensaje {
                    Message::Connect(_) => {
                        self.recibi_connect = true;
                        self.respuestas
                            .push(Respuesta::Ok(Some("connect".to_string())));
                    }
                    _ => {
                        self.respuestas.push(Respuesta::Err(
                            "Primero debe enviar un mensaje de conexión".to_string(),
                        ));
                    }
                }
                continue;
            }

            // proximo mensaje va a leer los bytes nuevos y devuelve si es una accion valida
            match mensaje {
                Message::Pub(subject, replay_to, payload) => {
                    println!("Publicacion: {:?} {:?} {:?}", subject, replay_to, payload);
                    self.publicaciones_salientes
                        .push(Publicacion::new(subject, payload, None, replay_to));
                    self.respuestas.push(Respuesta::Ok(Some("pub".to_string())));
                }
                Message::Hpub(subject, replay_to, headers, payload) => {
                    self.publicaciones_salientes.push(Publicacion::new(
                        subject,
                        payload,
                        Some(headers),
                        replay_to,
                    ));
                    self.respuestas
                        .push(Respuesta::Ok(Some("hpub".to_string())));
                }
                Message::Sub(topico, _, id) => match Subject::new(topico) {
                    Ok(sub) => {
                        self.subscripciones.insert(id, sub);
                        self.respuestas.push(Respuesta::Ok(Some("sub".to_string())));
                    }
                    Err(_) => {
                        self.respuestas.push(Respuesta::Err(
                            "Tópico de subscripción incorrecto".to_string(),
                        ));
                    }
                },
                Message::Unsub(subject, _) => {
                    self.subscripciones.remove(&subject);
                    self.respuestas
                        .push(Respuesta::Ok(Some("unsub".to_string())));
                }
                Message::Err(msg) => {
                    self.respuestas.push(Respuesta::Err(msg));
                }
                Message::Connect(_) => {
                    self.respuestas.push(Respuesta::Err(
                        "Ya se recibió un mensaje de conexión".to_string(),
                    ));
                }
                Message::Ping() => {
                    self.respuestas.push(Respuesta::Pong());
                }
            }
        }
    }

    /// Agrega un mensaje para que reciba al cliente
    ///
    /// Este método se encarga de filtrar el mensaje según las subscripciones que tenga el cliente
    pub fn recibir_mensaje(&mut self, publicacion: Publicacion) {
        for subject in self.subscripciones.values() {
            if subject.test(&publicacion.topico) {
                self.respuestas.push(Respuesta::Msg(publicacion));
                break;
            }
        }
    }

    /// Extrae las publicaciones salientes que se generaron en el último tick
    pub fn extraer_publicaciones_salientes(&mut self) -> Vec<Publicacion> {
        self.publicaciones_salientes.drain(..).collect() // drane saca los elementos del vector
    }

    /// Realiza una iteración de la conexión
    ///
    /// Este método se encarga de leer mensajes, enviar mensajes y procesar mensajes
    ///
    /// Se debe llamar a este método en un loop para que la conexión funcione
    ///
    /// Este método no bloquea, si no hay datos para leer o enviar, no hace nada
    ///
    /// Este método no maneja errores, si hay un error en la conexión, se debe manejar en el loop principal
    pub fn tick(&mut self) {
        let mut buffer = [0; 1024]; // 1kb
                                    // 1. Leer una vez
        match self.stream.read(&mut buffer) {
            Ok(n) => {
                // 2. Enviar bytes a parser y leer nuevos mensajes generados
                self.parser.agregar_bytes(&buffer[..n]);
                // 3. Leer mensajes
                self.leer_mensajes();
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No hay datos para leer (no hay que hacer nada acá)
            }
            Err(e) => {
                panic!(
                    "Error: {} (acá debería gestionar la desconexión probablemente)",
                    e
                );
            }
        }

        if self.respuestas.len() > 0 {
            println!("Respuestas: {:?}", &self.respuestas);
        }

        // 4. Enviar mensajes
        for mut respuesta in self.respuestas.drain(..) {
            let buffer = respuesta.serializar();
            if let Err(e) = self.stream.write_all(&buffer) {
                panic!(
                    "Error: {} (acá debería gestionar la desconexión probablemente)",
                    e
                );
            }
        }
    }
}

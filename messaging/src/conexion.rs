use std::{
    collections::HashMap,
    io::{self, Read, Write},
};

use crate::{configuracion::Configuracion, stream::Stream};

use super::{
    message::Message, parser::Parser, publicacion::Publicacion, respuesta::Respuesta,
    topico::Topico,
}; // Usamos super porque estamos en el mismo módulo

/// La conexión es responsable de mantener el stream con el cliente, leer mensajes y enviar mensajes
///
/// El proceso este se va realizando en el tiempo a través de llamar el método `tick` en un loop
pub struct Conexion {
    /// El stream de la conexión
    stream: Box<dyn Stream>,
    /// El parser se encarga de leer los bytes y generar mensajes
    parser: Parser,
    /// Por cada conexion, vamos a guardar los topicos a los que se suscribio
    suscripciones: HashMap<String, Topico>,
    /// Max msgs de suscripciones
    suscripciones_max_msgs: HashMap<String, u64>,
    /// Las publicaciones que manda el cliente
    publicaciones_salientes: Vec<Publicacion>,
    /// Las respuestas y publicaciones que se envian al stream del cliente
    respuestas: Vec<Respuesta>,
    /// Flag para saber si la conexión está activa
    recibi_connect: bool,
    /// Configuración
    configuracion: Configuracion,
    /// Si está desconectado
    pub desconectado: bool,
    /// Suscripciones salientes, el proceso las agarra para saber a quien se suscribio
    pub suscripciones_salientes: Vec<Topico>,
    /// Desuscripciones salientes, el proceso las agarra para saber a quien se desuscribio
    pub desuscripciones_salientes: Vec<Topico>,
}

impl Conexion {
    pub fn new(stream: Box<dyn Stream>, configuracion: Configuracion) -> Self {
        let respuestas = vec![Respuesta::Info()];
        Self {
            stream, // Los bytes de donde vamos a saber: QUE hay que hacer en DONDE, y si es publicar, el mensaje
            parser: Parser::new(),
            suscripciones: HashMap::new(),
            suscripciones_max_msgs: HashMap::new(),
            publicaciones_salientes: Vec::new(),
            respuestas,
            recibi_connect: false,
            configuracion,
            desconectado: false,
            suscripciones_salientes: Vec::new(),
            desuscripciones_salientes: Vec::new(),
        }
    }

    /// Agrega una nueva suscripción y le informa al proceso si hace falta agregarla
    pub fn nueva_suscripcion(&mut self, topico: Topico, id: String) {
        // Detectar si ya estaba suscrito el rópico
        let mut ya_estaba_suscrito = false;
        for (_, topico) in self.suscripciones.iter() {
            if topico.eq(topico) {
                ya_estaba_suscrito = true;
                break;
            }
        }

        // Si no estaba suscrito, agregarlo a las suscripciones salientes
        if !ya_estaba_suscrito {
            self.suscripciones_salientes.push(topico.clone());
        }

        // Agregar la suscripción
        self.suscripciones.insert(id, topico);
    }

    pub fn nueva_desuscripcion(&mut self, id: &str) {
        // Remover la suscripción
        if let Some(topico) = self.suscripciones.remove(id) {
            self.suscripciones_max_msgs.remove(id);

            // Detectar si sigue suscrito el tópico
            let mut sigue_suscrito = false;
            for (_, topico) in self.suscripciones.iter() {
                if topico.eq(topico) {
                    sigue_suscrito = true;
                    break;
                }
            }

            // Si no sigue suscrito, agregarlo a las desuscripciones salientes
            if !sigue_suscrito {
                self.desuscripciones_salientes.push(topico.clone());
            }
        }
    }

    /// Lee los mensajes nuevos recibidos del stream y que fueron previamente enviados al parser
    pub fn leer_mensajes(&mut self) {
        while let Some(mensaje) = self.parser.proximo_mensaje() {
            if self.configuracion.registros {
                // Devuelve que tipo de mensaje es
                println!("Mensaje: {:?}", mensaje);
            }

            if !self.recibi_connect {
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
                    if self.configuracion.registros {
                        println!("Publicacion: {:?} {:?} {:?}", subject, replay_to, payload);
                    }

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
                Message::Sub(topico, _, id) => match Topico::new(topico) {
                    Ok(topico) => {
                        self.nueva_suscripcion(topico, id);
                        self.respuestas.push(Respuesta::Ok(Some("sub".to_string())));
                    }
                    Err(_) => {
                        self.respuestas.push(Respuesta::Err(
                            "Tópico de subscripción incorrecto".to_string(),
                        ));
                    }
                },
                Message::Unsub(id, max_msgs) => {
                    if let Some(max) = max_msgs {
                        self.suscripciones_max_msgs.insert(id.clone(), max);
                    } else {
                        self.nueva_desuscripcion(&id);
                    }
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
        let mut desuscripciones = Vec::new();

        for (id, subject) in &self.suscripciones {
            if subject.test(&publicacion.topico) {
                if let Some(max_msgs) = self.suscripciones_max_msgs.get_mut(id) {
                    let nuevo_max_msgs = *max_msgs - 1;
                    if *max_msgs == 0 {
                        desuscripciones.push(id.clone());
                        self.suscripciones_max_msgs.remove(id);
                    } else {
                        self.suscripciones_max_msgs
                            .insert(id.clone(), nuevo_max_msgs);
                    }
                }

                self.respuestas
                    .push(Respuesta::Msg(publicacion.mensaje(id.to_owned())));
                break;
            }
        }

        for id in desuscripciones {
            self.nueva_desuscripcion(&id);
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
        if self.desconectado {
            return;
        }

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
            Err(_) => {
                self.desconectado = true;
                return;
            }
        }

        if !self.respuestas.is_empty() {
            println!("Respuestas: {:?}", &self.respuestas);
        }

        // 4. Enviar mensajes
        for mut respuesta in self.respuestas.drain(..) {
            let buffer = respuesta.serializar();
            if let Err(_) = self.stream.write_all(&buffer) {
                self.desconectado = true;
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use crate::{configuracion::Configuracion, mock_stream::MockStream, stream::Stream};

    use super::Conexion;

    #[test]
    fn test_connect() {
        let (escribir_bytes, rx) = mpsc::channel();
        let (tx, recibir_bytes) = mpsc::channel();
        let stream = MockStream::new(tx, rx);

        escribir_bytes.send(b"CONNECT {}\r\n".to_vec()).unwrap();

        let b: Box<dyn Stream> = Box::new(stream);

        let mut con = Conexion::new(b, Configuracion::new());

        con.tick();

        assert!(con.recibi_connect);

        let bytes = recibir_bytes.try_recv().unwrap();
        let txt = String::from_utf8_lossy(&bytes);
        assert!(txt.starts_with("INFO"));

        escribir_bytes.send(b"CONNECT {}\r\n".to_vec()).unwrap();

        con.tick();

        let bytes = recibir_bytes.try_recv().unwrap();
        let txt = String::from_utf8_lossy(&bytes);

        assert!(txt.starts_with("+OK"));
    }

    #[test]
    fn test_nueva_suscripcion() {
        let (escribir_bytes, rx) = mpsc::channel();
        let (tx, _recibir_bytes) = mpsc::channel();
        let stream = MockStream::new(tx, rx);

        escribir_bytes.send(b"CONNECT {}\r\n".to_vec()).unwrap();
        escribir_bytes.send(b"SUB asd 1\r\n".to_vec()).unwrap();

        let b: Box<dyn Stream> = Box::new(stream);

        let mut con = Conexion::new(b, Configuracion::new());

        con.tick();

        assert!(con.suscripciones.contains_key("1"));
        assert!(con.suscripciones.get("1").unwrap().to_string().eq("asd"));

        assert!(con.suscripciones_salientes.len() == 1);
        assert!(con.suscripciones_salientes[0].to_string().eq("asd"));
    }
}

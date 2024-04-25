use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::TcpStream,
};

use super::{message::Message, parser::Parser, publicacion::Publicacion}; // Usamos super porque estamos en el mismo módulo

/// La conexión es responsable de mantener el stream con el cliente, leer mensajes y enviar mensajes
///
/// El proceso este se va realizando en el tiempo a través de llamar el método `tick` en un loop
pub struct Conexion {
    stream: TcpStream,
    parser: Parser,
    subscripciones: HashMap<String, String>, // Por cada conexion, vamos a guardar los topicos a los que se suscribio
    publicaciones_salientes: Vec<Publicacion>, // Las publicaciones que manda el cliente
    publicaciones_entrantes: Vec<Publicacion>, // Las publicaciones que recibe el cliente
}

impl Conexion {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: stream, // Los bytes de donde vamos a saber: QUE hay que hacer en DONDE, y si es publicar, el mensaje
            parser: Parser::new(),
            subscripciones: HashMap::new(),
            publicaciones_salientes: Vec::new(),
            publicaciones_entrantes: Vec::new(),
        }
    }

    /// Lee los mensajes nuevos recibidos del stream y que fueron previamente enviados al parser
    pub fn leer_mensajes(&mut self) {
        while let Some(mensaje) = self.parser.proximo_mensaje() { // proximo mensaje va a leer los bytes nuevos y devuelve si es una accion valida
            match mensaje {
                Message::Sub(topico, id) => { 
                    self.subscripciones.insert(id, topico);
                }
                Message::Pub(topico, payload, replay_to) => self
                    .publicaciones_salientes
                    .push(Publicacion::new(topico, payload, replay_to)),
            }
        }
    }

    /// Agrega un mensaje para publicar al cliente
    /// 
    /// Este método se encarga de filtrar el mensaje según las subscripciones que tenga el cliente
    pub fn publicar(&mut self, publicacion: Publicacion) {
        // TODO: Verificar tópico
        self.publicaciones_entrantes.push(publicacion);
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
                // 2. Enviar bytes a parser
                self.parser.agregar_bytes(&buffer[..n]);
                // 3. Leer mensajes
                self.leer_mensajes();
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No hay datos para leer
            }
            Err(e) => {
                panic!("Error: {}", e);
            }
        }

        // 4. Enviar mensajes
        for publicacion in self.publicaciones_entrantes.drain(..) {
            let buffer = publicacion.serializar();
            self.stream.write_all(&buffer).unwrap();
        }
    }
}

use std::{
    collections::HashMap,
    io::{self, Read, Write},
    sync::mpsc::{Receiver, Sender},
};

use lib::{
    parseador::{mensaje::Mensaje, Parseador},
    stream::Stream,
};

use super::{instruccion::Instruccion, publicacion::Publicacion};

/// El hilo del cliente posee el stream de la conexion, el canal por el cual se
/// reciben mensajes, los canales de suscripciones que están asociados a un id
/// de suscripción, y el Parseador
pub struct HiloCliente {
    pub stream: Box<dyn Stream>,
    pub canal_recibir: Receiver<Instruccion>,
    // Cada canal de cada subscripción está asociado a un id de subscripción
    pub canales_subscripciones: HashMap<String, Sender<Publicacion>>,
    pub autenticado: bool,
    parseador: Parseador,
}

impl HiloCliente {
    pub fn new(stream: Box<dyn Stream>, canal_recibir: Receiver<Instruccion>) -> Self {
        Self {
            stream,
            canal_recibir,
            canales_subscripciones: HashMap::new(),
            parseador: Parseador::new(),
            autenticado: false,
        }
    }

    pub fn ejecutar(&mut self) -> std::io::Result<()> {
        loop {
            if !self.ciclo()? {
                break;
            }
        }

        Ok(())
    }

    fn ciclo(&mut self) -> std::io::Result<bool> {
        let mut conectado = false;

        while let Some(mensaje) = self.proximo_mensaje()? {
            self.gestionar_nuevo_mensaje(mensaje)?;
        }

        // Esperar a que termine de autenticarse para procesar instrucciones
        if !self.autenticado {
            return Ok(conectado);
        }

        while let Ok(instruccion) = self.canal_recibir.try_recv() {
            conectado = self.gestionar_nueva_instruccion(instruccion)?;
        }

        Ok(conectado)
    }

    fn gestionar_nuevo_mensaje(&mut self, mensaje: Mensaje) -> std::io::Result<()> {
        match mensaje {
            // Ejemplo: MSG 1 4\r\nhola\r\n
            Mensaje::Publicacion(topico, id_suscripcion, responder_a, contenido) => {
                let publicacion = Publicacion {
                    header: None,
                    payload: contenido,
                    replay_to: responder_a,
                    subject: topico,
                };

                if let Some(canal) = self.canales_subscripciones.get(&id_suscripcion) {
                    if let Err(e) = canal.send(publicacion) {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
                    }
                }
            }
            // Ejemplo: INFO {"server_id":"a","version":"2.1.0","go":"go1.15.6","host":"...
            Mensaje::Info() => {
                self.stream.write_all(b"CONNECT {}\r\n")?;
                self.autenticado = true;
            }
            // Ejemplo: PING\r\n
            Mensaje::Ping() => {
                self.stream.write_all(b"PONG\r\n")?;
            }
            _ => {
                eprintln!("Mensaje no reconocido: {:?}", mensaje)
            }
        }

        Ok(())
    }

    fn gestionar_nueva_instruccion(&mut self, instruccion: Instruccion) -> std::io::Result<bool> {
        match instruccion {
            Instruccion::Suscribir {
                id_suscripcion,
                canal,
                queue_group,
                topico,
            } => {
                self.canales_subscripciones
                    .insert(id_suscripcion.to_owned(), canal);

                if let Some(queue_group) = queue_group {
                    self.stream.write_all(
                        format!("SUB {} {} {}\r\n", topico, id_suscripcion, queue_group).as_bytes(),
                    )?;
                } else {
                    self.stream
                        .write_all(format!("SUB {} {}\r\n", topico, id_suscripcion).as_bytes())?;
                }
            }
            Instruccion::Desuscribir { id_suscripcion } => {
                self.canales_subscripciones
                    .remove(&id_suscripcion.to_string());
                self.stream
                    .write_all(format!("UNSUB {}\r\n", id_suscripcion).as_bytes())?;
            }
            Instruccion::Publicar(publicacion) => {
                if let Some(reply_to) = publicacion.replay_to {
                    if let Some(header) = &publicacion.header {
                        self.stream.write_all(
                            format!(
                                "PUB {} {} {} {}\r\n",
                                publicacion.subject,
                                reply_to,
                                header.len(),
                                publicacion.payload.len()
                            )
                            .as_bytes(),
                        )?;
                        self.stream.write_all(header)?;
                        self.stream.write_all(b"\r\n")?;
                        self.stream.write_all(&publicacion.payload)?;
                        self.stream.write_all(b"\r\n")?;
                    } else {
                        self.stream.write_all(
                            format!(
                                "PUB {} {} {}\r\n",
                                publicacion.subject,
                                reply_to,
                                publicacion.payload.len()
                            )
                            .as_bytes(),
                        )?;
                        self.stream.write_all(&publicacion.payload)?;
                        self.stream.write_all(b"\r\n")?;
                    }
                } else if let Some(header) = &publicacion.header {
                    self.stream.write_all(
                        format!(
                            "HPUB {} {} {}\r\n",
                            publicacion.subject,
                            header.len(),
                            publicacion.payload.len()
                        )
                        .as_bytes(),
                    )?;
                    self.stream.write_all(header)?;
                    self.stream.write_all(b"\r\n")?;
                    self.stream.write_all(&publicacion.payload)?;
                    self.stream.write_all(b"\r\n")?;
                } else {
                    self.stream.write_all(
                        format!(
                            "PUB {} {}\r\n",
                            publicacion.subject,
                            publicacion.payload.len()
                        )
                        .as_bytes(),
                    )?;
                    self.stream.write_all(&publicacion.payload)?;
                    self.stream.write_all(b"\r\n")?;
                }
            }
            Instruccion::Desconectar => {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn proximo_mensaje(&mut self) -> std::io::Result<Option<Mensaje>> {
        let mut buffer = [0; 1024];

        match self.stream.read(&mut buffer) {
            Ok(n) => {
                self.parseador.agregar_bytes(&buffer[..n]);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No hay datos para leer (no hay que hacer nada acá)
            }
            Err(e) => {
                return Err(e);
            }
        }
        Ok(self.parseador.proximo_mensaje())
    }
}

use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};

use crate::{instruccion::Instruccion, mensaje::Mensaje, publicacion::Publicacion};

pub struct HiloCliente {
    pub stream: TcpStream,
    pub canal_recibir: Receiver<Instruccion>,
    // Cada canal de cada subscripción está asociado a un id de subscripción
    pub canales_subscripciones: HashMap<String, Sender<Publicacion>>,
}

impl HiloCliente {
    pub fn new(stream: TcpStream, canal_recibir: Receiver<Instruccion>) -> Self {
        Self {
            stream,
            canal_recibir,
            canales_subscripciones: HashMap::new(),
        }
    }

    pub fn ejecutar(&mut self) -> std::io::Result<()> {
        let mut desconectar = false;

        self.stream.write_all(b"CONNECT {}\r\n")?;

        while !desconectar {
            if let Some(mensaje) = self.proximo_mensaje()? {
                self.gestionar_nuevo_mensaje(mensaje)?;
            }

            if let Ok(instruccion) = self.canal_recibir.try_recv() {
                desconectar = self.gestionar_nueva_instruccion(instruccion)?;
            }
        }

        Ok(())
    }

    fn gestionar_nuevo_mensaje(&mut self, mensaje: Mensaje) -> std::io::Result<()> {
        match mensaje {
            Mensaje::Publicacion(sid, publicacion) => {
                if let Some(canal) = self.canales_subscripciones.get(&sid) {
                    if let Err(e) = canal.send(publicacion) {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn gestionar_nueva_instruccion(&mut self, instruccion: Instruccion) -> std::io::Result<bool> {
        match instruccion {
            Instruccion::Subscribir {
                id_subscripcion,
                canal,
                queue_group,
                topico,
            } => {
                self.canales_subscripciones
                    .insert(id_subscripcion.to_owned(), canal);
                if let Some(queue_group) = queue_group {
                    self.stream.write_all(
                        format!("SUB {} {} {}\r\n", topico, id_subscripcion, queue_group)
                            .as_bytes(),
                    )?;
                } else {
                    self.stream
                        .write_all(format!("SUB {} {}\r\n", topico, id_subscripcion).as_bytes())?;
                }
            }
            Instruccion::Desubscribir { id_subscripcion } => {
                self.canales_subscripciones
                    .remove(&id_subscripcion.to_string());
                self.stream
                    .write_all(format!("UNSUB {}\r\n", id_subscripcion).as_bytes())?;
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
                        self.stream.write_all(&header)?;
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
                } else {
                    if let Some(header) = &publicacion.header {
                        self.stream.write_all(
                            format!(
                                "PUB {} {} {}\r\n",
                                publicacion.subject,
                                header.len(),
                                publicacion.payload.len()
                            )
                            .as_bytes(),
                        )?;
                        self.stream.write_all(&header)?;
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
            }
            Instruccion::Desconectar => {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn proximo_mensaje(&mut self) -> std::io::Result<Option<Mensaje>> {
        let mut buffer = [0; 1024];
        match self.stream.read(&mut buffer) {
            Ok(n) => {
                todo!()
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No hay datos para leer (no hay que hacer nada acá)
            }
            Err(e) => {
                return Err(e);
            }
        }
        Ok(None)
    }
}

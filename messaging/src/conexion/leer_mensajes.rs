use crate::{publicacion::Publicacion, suscripciones::suscripcion::Suscripcion, topico::Topico};

use super::{message::Message, respuesta::Respuesta, tick_contexto::TickContexto, Conexion};

/// Lee los mensajes nuevos recibidos del stream y que fueron previamente enviados al parser
pub fn leer_mensajes(conexion_self: &mut Conexion, contexto: &mut TickContexto) {
    while let Some(mensaje) = conexion_self.parser.proximo_mensaje() {
        conexion_self.registrador.info(
            &format!("Mensaje recibido: {:?}", mensaje),
            Some(conexion_self.id),
        );

        if !conexion_self.autenticado {
            match mensaje {
                Message::Connect(_) => {
                    conexion_self.autenticado = true;
                    conexion_self.escribir_respuesta(&Respuesta::Ok(Some("connect".to_string())));
                }
                _ => {
                    conexion_self.escribir_err(Some(
                        "Primero debe enviar un mensaje de conexión".to_string(),
                    ));
                    conexion_self.desconectado = true;
                    return;
                }
            }
            continue;
        }

        // proximo mensaje va a leer los bytes nuevos y devuelve si es una accion valida
        match mensaje {
            Message::Pub(subject, replay_to, payload) => {
                conexion_self.registrador.info(
                    &format!("Publicación: {:?} {:?} {:?}", subject, replay_to, payload),
                    Some(conexion_self.id),
                );

                contexto.publicar(Publicacion::new(subject, payload, None, replay_to));
                conexion_self.escribir_ok(Some("pub".to_string()));
            }
            Message::Hpub(subject, replay_to, headers, payload) => {
                conexion_self.registrador.info(
                    &format!(
                        "Publicación con header: {:?} {:?} {:?} {:?}",
                        subject, headers, replay_to, payload
                    ),
                    Some(conexion_self.id),
                );

                contexto.publicar(Publicacion::new(subject, payload, Some(headers), replay_to));
                conexion_self.escribir_ok(Some("hpub".to_string()));
            }
            Message::Sub(topico, grupo, id) => match Topico::new(topico) {
                Ok(topico) => {
                    contexto.suscribir(Suscripcion::new(
                        contexto.id_hilo,
                        conexion_self.id,
                        topico,
                        id,
                        grupo,
                    ));
                    conexion_self.escribir_ok(Some("sub".to_string()));
                }
                Err(_) => {
                    conexion_self
                        .escribir_err(Some("Tópico de subscripción incorrecto".to_string()));
                }
            },
            Message::Unsub(id, max_msgs) => {}
            Message::Err(msg) => {
                // conexion_self.respuestas.push(Respuesta::Err(msg));
                conexion_self.escribir_err(Some(msg));
            }
            Message::Connect(_) => {
                conexion_self
                    .escribir_err(Some("Ya se recibió un mensaje de conexión".to_string()));
            }
            Message::Ping() => {
                conexion_self.escribir_respuesta(&Respuesta::Pong());
            }
        }
    }
}

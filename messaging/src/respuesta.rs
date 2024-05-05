use crate::publicacion_mensaje::PublicacionMensaje;

#[derive(Debug)]
pub enum Respuesta {
    Msg(PublicacionMensaje),
    Err(String),
    Ok(Option<String>),
    Pong(),
    Info(),
}

impl Respuesta {
    pub fn serializar(&mut self) -> Vec<u8> {
        match self {
            Respuesta::Msg(publicacion) => publicacion.serializar_msg(),
            Respuesta::Err(error) => {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(b"-ERR ");
                bytes.extend_from_slice(error.as_bytes());
                bytes.extend_from_slice(b"\r\n");
                bytes
            }
            Respuesta::Ok(msg) => {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(b"+OK");
                if let Some(msg) = msg {
                    bytes.extend_from_slice(b" ");
                    bytes.extend_from_slice(msg.as_bytes());
                }
                bytes.extend_from_slice(b"\r\n");
                bytes
            }
            Respuesta::Pong() => {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(b"PONG");
                bytes.extend_from_slice(b"\r\n");
                bytes
            }
            Respuesta::Info() => {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(b"INFO {}");
                bytes.extend_from_slice(b"\r\n");
                bytes
            }
        }
    }
}

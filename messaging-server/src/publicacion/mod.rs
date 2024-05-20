use self::mensaje::PublicacionMensaje;

pub mod mensaje;

/// Representa un mensaje que se va a publicar en un tópico
#[derive(Debug, Clone)]
pub struct Publicacion {
    pub topico: String,            // A donde se envia el mensaje
    pub payload: Vec<u8>,          // El mensaje que se va a enviar
    pub header: Option<Vec<u8>>,   // EL header del mensaje que se va a enviar
    pub replay_to: Option<String>, // Campo que tiene nats
}

impl Publicacion {
    pub fn new(
        topico: String,
        payload: Vec<u8>,
        header: Option<Vec<u8>>,
        replay_to: Option<String>,
    ) -> Self {
        Self {
            topico,
            payload,
            replay_to,
            header,
        }
    }

    pub fn mensaje(&self, sid: String) -> PublicacionMensaje {
        PublicacionMensaje::new(
            sid,
            self.topico.clone(),
            self.payload.clone(),
            self.header.clone(),
            self.replay_to.clone(),
        )
    }
}

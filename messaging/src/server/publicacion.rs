/// Representa un mensaje que se va a publicar en un tópico
#[derive(Debug)]
pub struct Publicacion {
    pub topico: String,            // A donde se envia el mensaje
    pub payload: Vec<u8>,          // El mensaje que se va a enviar
    pub headers: Option<Vec<u8>>,  // Los headers del mensaje que se va a enviar
    pub replay_to: Option<String>, // Campo que tiene nats
}

impl Publicacion {
    pub fn new(
        topico: String,
        payload: Vec<u8>,
        headeres: Option<Vec<u8>>,
        replay_to: Option<String>,
    ) -> Self {
        Self {
            topico: topico,
            payload: payload,
            replay_to: replay_to,
            headers: headeres,
        }
    }

    pub fn serializar_msg(&self) -> Vec<u8> {
        // MSG <subject> <sid> [reply-to] <#bytes>␍␊[payload]␍␊

        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"MSG ");
        bytes.extend_from_slice(self.topico.as_bytes());
        bytes.extend_from_slice(b" ");
        if let Some(replay_to) = &self.replay_to {
            bytes.extend_from_slice(replay_to.as_bytes());
        }
        bytes.extend_from_slice(b" ");
        bytes.extend_from_slice(self.payload.len().to_string().as_bytes());
        bytes.extend_from_slice(b"\r\n");
        bytes.extend_from_slice(&self.payload);
        bytes.extend_from_slice(b"\r\n");

        bytes
    }
}

impl Clone for Publicacion {
    fn clone(&self) -> Self {
        Self {
            topico: self.topico.clone(),
            payload: self.payload.clone(),
            replay_to: self.replay_to.clone(),
            headers: self.headers.clone(),
        }
    }
}

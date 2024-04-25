/// Representa un mensaje que se va a publicar en un tópico
pub struct Publicacion {
    topico: String, // a donde se envia el mensaje
    payload: Vec<u8>, // El mensaje que se va a enviar
    replay_to: Option<String>, // Campo que tiene nats
}

impl Publicacion {
    pub fn new(topico: String, payload: Vec<u8>, replay_to: Option<String>) -> Self {
        Self {
            topico: topico,
            payload: payload,
            replay_to: replay_to,
        }
    }

    pub fn serializar(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // TODO: Serializar mensaje

        bytes
    }
}

impl Clone for Publicacion {
    fn clone(&self) -> Self {
        Self {
            topico: self.topico.clone(),
            payload: self.payload.clone(),
            replay_to: self.replay_to.clone(),
        }
    }
}

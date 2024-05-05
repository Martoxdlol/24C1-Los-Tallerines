use std::sync::mpsc::Sender;

use crate::publicacion::Publicacion;

// Instrucciones posibles que puede realizar el cliente
pub enum Instruccion {
    Publicar(Publicacion),
    Suscribir {
        topico: String,
        id_suscripcion: String,
        queue_group: Option<String>,
        canal: Sender<Publicacion>,
    },
    Desuscribir {
        id_suscripcion: String,
    },
    Desconectar,
}

use std::sync::mpsc::Sender;

use crate::publicacion::Publicacion;

pub enum Instruccion {
    Publicar(Publicacion),
    Subscribir {
        topico: String,
        id_suscripcion: String,
        queue_group: Option<String>,
        canal: Sender<Publicacion>,
    },
    Desubscribir {
        id_suscripcion: String,
    },
    Desconectar,
}

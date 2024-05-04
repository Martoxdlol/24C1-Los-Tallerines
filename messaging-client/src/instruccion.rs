use std::sync::mpsc::Sender;

use crate::publicacion::Publicacion;

pub enum Instruccion {
    Publicar(Publicacion),
    Subscribir {
        topico: String,
        id_subscripcion: String,
        queue_group: Option<String>,
        canal: Sender<Publicacion>,
    },
    Desubscribir {
        id_subscripcion: String,
    },
    Desconectar,
}

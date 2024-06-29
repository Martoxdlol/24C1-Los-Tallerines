use std::sync::mpsc::{Receiver, Sender};

use lib::{camara::Camara, deteccion::Deteccion};

pub fn hilo_deteccion_camara(
    camara: Camara,
    ruta: String,
    enviar_deteccion: Sender<Deteccion>,
    detener_deteccion: Receiver<()>,
) {
    // leer la ruta si hay algo
    // Si hay algo
    // pasarle al sistema detalle y ubicacion
}

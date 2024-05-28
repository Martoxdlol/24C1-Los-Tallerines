pub mod comando;
pub mod respuesta;
use std::sync::mpsc::{self, Receiver, Sender};

use self::{comando::Comando, respuesta::Respuesta};

/// Inicializa la terminal como interfaz de usuario. Devuelve un par de canales para enviar comandos y recibir respuestas.
pub fn interfaz() -> (Sender<Respuesta>, Receiver<Comando>) {

    let (tx1, rx1) = mpsc::channel::<Respuesta>();
    let (tx2, rx2) = mpsc::channel::<Comando>();

    (tx1, rx2)
}

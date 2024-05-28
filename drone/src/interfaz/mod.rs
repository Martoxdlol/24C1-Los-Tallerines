pub mod comando;
pub mod respuesta;
use std::{
    io,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use self::{comando::Comando, respuesta::Respuesta};

/// Inicializa la terminal como interfaz de usuario. Devuelve un par de canales para enviar comandos y recibir respuestas.
pub fn interfaz() -> (Sender<Respuesta>, Receiver<Comando>) {
    let (enviar_comando, recibir_comandos) = mpsc::channel::<Comando>();
    let (enviar_respuesta, recibir_respuestas) = mpsc::channel::<Respuesta>();

    thread::spawn(move || loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        println!("Por ahora no hay comando");
        std::process::exit(1);
    });

    (enviar_respuesta, recibir_comandos)
}

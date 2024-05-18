use std::{
    io,
    sync::mpsc::{Receiver, Sender},
};

use messaging_client::cliente::Cliente;

use crate::{
    estado::Estado,
    interfaz::{comando::Comando, respuesta::Respuesta},
};

pub struct Sistema {
    pub estado: Estado,
    enviar_respuesta: Sender<Respuesta>,
    recibir_comandos: Receiver<Comando>,
}

impl Sistema {
    pub fn new(
        estado: Estado,
        enviar_respuesta: Sender<Respuesta>,
        recibir_comandos: Receiver<Comando>,
    ) -> Self {
        Self {
            estado,
            enviar_respuesta,
            recibir_comandos,
        }
    }

    /// Inicia el bucle infinito del sistema
    ///
    /// Está función se encarga de reintentar la ejecución del sistema en caso de error.
    pub fn iniciar(&mut self) {
        loop {
            if let Err(e) = self.inicio() {
                eprintln!("Error en hilo principal: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }

    /// Inicia el bucle de eventos del sistema
    ///
    /// Este bucle puede terminar por un error de conexión
    pub fn inicio(&mut self) -> io::Result<()> {
        // Conectar el cliente al servidor de NATS
        let cliente = self.conectar()?;

        // Publicar al servidor de NATS el estado de todas las cámaras
        self.publicar_estado_general()?;

        loop {
            self.ciclo()?;
        }
    }

    /// Conectar el cliente
    fn conectar(&self) -> io::Result<Cliente> {
        Cliente::conectar("127.0.0.1:4222")
    }

    fn publicar_estado_general(&mut self) -> io::Result<()> {
        todo!()
    }

    pub fn ciclo(&mut self) -> io::Result<()> {
        todo!()
    }
}

use std::io;

use messaging_client::cliente::Cliente;

use crate::estado::Estado;

pub struct Sistema {
    pub estado: Estado,
    enviar_respuesta: &Sender<Respuesta>,
    recibir_comandos: &Receiver<Comando>,
}

impl Sistema {
    pub fn new(
        estado: Estado,
        enviar_respuesta: &Sender<Respuesta>,
        recibir_comandos: &Receiver<Comando>,
    ) -> Self {
        Self {
            estado,
            enviar_respuesta,
            recibir_comandos,
        }
    }

    pub fn iniciar(&mut self) {
        loop {
            if let Err(e) = self.inicio() {
                eprintln!("Error en hilo principal: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }

    pub fn inicio(&mut self) -> io::Result<()> {
        let cliente = self.conectar()?;

        loop {}
    }

    /// Conectar el cliente
    pub fn conectar() -> io::Result<Cliente> {
        Cliente::conectar("127.0.0.1:4222")
    }

    pub fn ciclo(&mut self) {
        
    }
}

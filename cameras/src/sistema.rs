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
    fn inicio(&mut self) -> io::Result<()> {
        // Conectar el cliente al servidor de NATS
        let cliente = self.conectar()?;

        // Publicar al servidor de NATS el estado de todas las cámaras
        self.publicar_estado_general()?;

        loop {
            self.ciclo(&cliente)?;
        }
    }

    /// Conectar el cliente
    fn conectar(&self) -> io::Result<Cliente> {
        Cliente::conectar("127.0.0.1:4222")
    }

    fn publicar_estado_general(&mut self) -> io::Result<()> {
        todo!()
    }

    fn ciclo(&mut self, cliente: &Cliente) -> io::Result<()> {
        self.leer_incidentes(cliente)?;
        self.leer_comandos(cliente)?;
        Ok(())
    }

    fn leer_incidentes(&mut self, cliente: &Cliente) -> io::Result<()> {
        todo!()
    }

    fn leer_comandos(&mut self, cliente: &Cliente) -> io::Result<()> {
        while let Ok(comando) = self.recibir_comandos.try_recv() {
            match comando {
                Comando::Conectar(id, lat, lon, rango) => {
                    self.comando_conectar_camara(cliente, id, lat, lon, rango)?
                }
                Comando::Desconectar(id) => self.comando_desconectar_camara(cliente, id)?,
                Comando::ListarCamaras => self.comando_listar_camaras(cliente)?,
                Comando::ModifciarRango(id, rango) => {
                    self.comando_modificar_rango(cliente, id, rango)?
                }
                Comando::ModificarUbicacion(id, lat, lon) => {
                    self.comando_modificar_ubicacion(cliente, id, lat, lon)?
                }
                Comando::Camara(id) => self.comando_mostrar_camara(cliente, id)?,
                Comando::Ayuda => self.comando_ayuda(cliente)?,
            }
        }

        Ok(())
    }

    fn comando_conectar_camara(
        &mut self,
        cliente: &Cliente,
        id: u64,
        lat: f64,
        lon: f64,
        rango: f64,
    ) -> io::Result<()> {
        todo!()
    }

    fn comando_desconectar_camara(&mut self, cliente: &Cliente, id: u64) -> io::Result<()> {
        todo!()
    }

    fn comando_listar_camaras(&mut self, cliente: &Cliente) -> io::Result<()> {
        todo!()
    }

    fn comando_modificar_rango(
        &mut self,
        cliente: &Cliente,
        id: u64,
        rango: f64,
    ) -> io::Result<()> {
        todo!()
    }

    fn comando_modificar_ubicacion(
        &mut self,
        cliente: &Cliente,
        id: u64,
        lat: f64,
        lon: f64,
    ) -> io::Result<()> {
        todo!()
    }

    fn comando_mostrar_camara(&mut self, cliente: &Cliente, id: u64) -> io::Result<()> {
        todo!()
    }

    fn comando_ayuda(&mut self, cliente: &Cliente) -> io::Result<()> {
        todo!()
    }
}

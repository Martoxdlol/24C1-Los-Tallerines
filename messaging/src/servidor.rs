use std::{
    collections::HashMap,
    io,
    net::TcpListener,
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
};

use crate::configuracion::Configuracion;

use super::{conexion::Conexion, proceso::Proceso};

pub struct Servidor {
    hilos: Vec<(Sender<(u64, Conexion)>, JoinHandle<()>)>,
    configuracion: Configuracion,
    i: usize,
}

impl Servidor {
    pub fn procesos(cantidad: usize) -> Servidor {
        let mut procesos = Vec::new();

        let mut canales_enviar = Vec::new();
        let mut canales_recibir = Vec::new();

        for _ in 0..cantidad {
            let (tx, rx) = mpsc::channel();
            canales_enviar.push(tx);
            canales_recibir.push(rx);
        }

        for (i, rx) in canales_recibir.drain(..).enumerate() {
            let mut otros_canales = HashMap::new();

            for (canal_i, tx) in canales_enviar.iter().enumerate() {
                if canal_i == i {
                    continue;
                }

                let id = canal_i as u64;

                otros_canales.insert(id, tx.clone());
            }

            let id: u64 = i as u64;

            let (tx_conexiones, rx_conexiones) = mpsc::channel();
            let proceso = Proceso::new(id, otros_canales, rx, rx_conexiones);

            let handle = Proceso::iniciar(proceso);
            procesos.push((tx_conexiones, handle));
        }

        Servidor {
            hilos: procesos,
            configuracion: Configuracion::new(),
            i: 0,
        }
    }

    pub fn iniciar(mut servidor: Servidor) -> JoinHandle<()> {
        thread::spawn(move || {
            servidor.inicio();
        })
    }

    pub fn inicio(&mut self) {
        let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
        listener
            .set_nonblocking(true) // Hace que el listener no bloquee el hilo principal
            .expect("No se pudo poner el listener en modo no bloqueante");

        let mut id: u64 = 0;

        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    stream.set_nonblocking(true).unwrap();
                    let conexion = Conexion::new(Box::new(stream), self.configuracion.clone()); // Si escucho algo, genero una nueva conexion

                    id += 1;
                    let nuevo_id = id;

                    let (tx, _) = &self.hilos[self.i];
                    match tx.send((nuevo_id,conexion)) {
                        Ok(_) => {
                            self.i = (self.i + 1) % self.hilos.len();
                        }
                        Err(e) => {
                            panic!("Error: {}", e);
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No hay conexiones nuevas
                }
                Err(e) => {
                    panic!("Error: {}", e);
                }
            }
        }
    }
}

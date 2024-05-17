use std::{
    collections::HashMap,
    io,
    net::TcpListener,
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
};

use crate::{
    conexion::id::IdConexion, configuracion::Configuracion, hilo::id::IdHilo,
    registrador::Registrador,
};

use super::{conexion::Conexion, hilo::Hilo};

type InfoHilo = (Sender<(IdConexion, Conexion)>, JoinHandle<()>);

pub struct Servidor {
    hilos: Vec<InfoHilo>,
    _configuracion: Configuracion,
    proximo_id_hilo: usize, // Cada conexión que se genera hay que asignarla a un hilo. Con esto determino a que hilo se lo doy. Si ponemos IdHilo no sirve como indice para Vec, pero si se puede convertir usize a IdHilo
    ultimo_id_conexion: IdConexion, // Cada id tiene que ser único por cada conexion. Se incrementa cada vez que se crea una nueva conexion
    registrador: Registrador,
}

impl Servidor {
    pub fn procesos(cantidad: usize) -> Servidor {
        // La cantidad es la cantidad de hilos que se van a crear
        // Vector con los canales para enviar nuevas conexiones y handle de los threads
        let mut hilos = Vec::new();

        // Puntas emisoras de los canales para enviar mensajes a los hilos
        let mut canales_enviar = Vec::new();
        // Puntas receptoras de los canales para recibir mensajes de los hilos
        let mut canales_recibir = Vec::new();

        // `logger`
        let registrador = Registrador::new();

        // Creamos los canales para enviar y recibir instrucciones entre los hilos
        for _ in 0..cantidad {
            let (tx, rx) = mpsc::channel();
            canales_enviar.push(tx);
            canales_recibir.push(rx);
        }

        for (indice_hilo, rx) in canales_recibir.drain(..).enumerate() {
            // HashMap con las puntas emisoras a cada hilo para enviar instrucciones a los mismos
            let mut canales_a_enviar_mensajes = HashMap::new();

            // Insertamos las puntas emisoras de los canales en el HashMap
            for (id_canal_a_enviar, tx) in canales_enviar.iter().enumerate() {
                let id = id_canal_a_enviar as IdHilo;
                canales_a_enviar_mensajes.insert(id, tx.clone()); // El id es el id del hilo. Yo quiero mandarle mensaje a todos los hilos.
                                                                  // a cada id, le asigno un emisor a ese hilo. (id 2, le asigno un emisor al hilo 2)
            }

            // Obtengo el id del hilo
            let id_hilo: u64 = indice_hilo as IdHilo; // Id del hilo actual. Suponiendo cronologia; 1, 2...

            // Creamos el canal para enviar nuevas conexiones al hilo
            let (tx_conexiones, rx_conexiones) = mpsc::channel();
            // Creamos el registrador para el hilo
            let mut registrador = registrador.clone();
            // Establecemos el hilo actual para el registrador
            registrador.establecer_hilo(id_hilo);
            // Creamos el hilo
            let hilo = Hilo::new(
                id_hilo,
                rx_conexiones,
                canales_a_enviar_mensajes,
                rx,
                registrador,
            );

            // Iniciamos el thread del hilo
            let handle = Hilo::iniciar(hilo);
            // Tx_conexciones es por donde le van a asignar conexiones al hilo y el handle del hilo
            hilos.push((tx_conexiones, handle));
        }

        Servidor {
            hilos,
            _configuracion: Configuracion::new(),
            proximo_id_hilo: 0,
            ultimo_id_conexion: 0,
            registrador,
        }
    }

    pub fn nuevo_id_conexion(&mut self) -> IdConexion {
        self.ultimo_id_conexion += 1;
        self.ultimo_id_conexion
    }

    pub fn iniciar(mut servidor: Servidor) -> JoinHandle<()> {
        thread::spawn(move || {
            servidor.inicio();
        })
    }

    pub fn inicio(&mut self) {
        let listener = TcpListener::bind("0.0.0.0:4222").unwrap();
        listener
            .set_nonblocking(true) // Hace que el listener no bloquee el hilo principal
            .expect("No se pudo poner el listener en modo no bloqueante");

        loop {
            match listener.accept() {
                // Si escucho algo, genero una nueva conexion
                Ok((stream, _)) => {
                    stream.set_nonblocking(true).unwrap();

                    // Creamos una copia del logger para la nueva conexion
                    let mut registrador_para_nueva_conexion = self.registrador.clone();
                    // Establecemos el hilo actual para la nueva conexion
                    registrador_para_nueva_conexion.establecer_hilo(self.proximo_id_hilo as IdHilo);

                    // Generamos un nuevo id único para la nueva conexión
                    let id_conexion = self.nuevo_id_conexion();

                    let conexion = Conexion::new(
                        id_conexion,
                        Box::new(stream),
                        registrador_para_nueva_conexion,
                    );

                    let (tx, _) = &self.hilos[self.proximo_id_hilo];
                    match tx.send((id_conexion, conexion)) {
                        // Envio la conexion al hilo
                        Ok(_) => {
                            self.proximo_id_hilo = (self.proximo_id_hilo + 1) % self.hilos.len();
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

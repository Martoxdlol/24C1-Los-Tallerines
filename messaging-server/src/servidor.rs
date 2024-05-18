use std::{
    collections::HashMap,
    io,
    net::TcpListener,
    sync::{
        mpsc::{self, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crate::{
    conexion::id::IdConexion, configuracion::Configuracion, cuenta::Cuenta, hilo::id::IdHilo,
    registrador::Registrador,
};

use super::{conexion::Conexion, hilo::Hilo};

use serde::Deserialize;
use std::env;
use std::fs;
use std::process;

#[derive(Deserialize)]
struct Config {
    direccion: String,
    puerto: u16,
}

type InfoHilo = (Sender<(IdConexion, Conexion)>, JoinHandle<()>);

pub struct Servidor {
    hilos: Vec<InfoHilo>,
    _configuracion: Configuracion,
    proximo_id_hilo: usize, // Cada conexión que se genera hay que asignarla a un hilo. Con esto determino a que hilo se lo doy. Si ponemos IdHilo no sirve como indice para Vec, pero si se puede convertir usize a IdHilo
    ultimo_id_conexion: IdConexion, // Cada id tiene que ser único por cada conexion. Se incrementa cada vez que se crea una nueva conexion
    registrador: Registrador,
    cuentas: Arc<Vec<Cuenta>>,
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

        // Para cada punta receptora en canales_recibir, se insertan las
        // puntas emisoras de los canales en canales_a_enviar_mensajes que
        // tiene las puntas emisoras a cada hilo para enviar instrucciones
        // a ellos
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
                rx_conexiones,             // punta receptora para recibir conexiones
                canales_a_enviar_mensajes, // punta emisora para enviar instrucciones
                rx,                        // punta receptora para recibir instrucciones
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
            cuentas: Arc::new(Vec::new()),
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

    fn obtener_configuracion(&mut self) -> Config {
        let argumentos: Vec<String> = env::args().collect();

        if argumentos.len() != 2 {
            println!("No se mandó archivo de configuración");
            process::exit(1);
        }

        let path_archivo = &argumentos[1];

        let contenido = fs::read_to_string(path_archivo).unwrap_or_else(|err| {
            eprintln!("No se pudo leer el archivo de configuración: {}", err);
            process::exit(1);
        });

        let configuracion: Config = toml::from_str(&contenido).unwrap_or_else(|err| {
            eprintln!("No se pudo parsear el archivo de configuración: {}", err);
            process::exit(1);
        });

        configuracion
    }

    pub fn inicio(&mut self) {
        let configuracion = self.obtener_configuracion();

        let listener = TcpListener::bind(format!(
            "{}:{}",
            configuracion.direccion, configuracion.puerto
        ))
        .unwrap();

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
                        Some(self.cuentas.clone()),
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

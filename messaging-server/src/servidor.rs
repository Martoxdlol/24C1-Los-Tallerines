use std::{
    collections::HashMap,
    io,
    net::TcpListener,
    sync::{
        mpsc::{self, channel, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
};

use lib::{configuracion::Configuracion, stream::Stream};
use native_tls::{Identity, TlsAcceptor};

use crate::{
    conexion::{id::IdConexion, r#trait::Conexion},
    cuenta::Cuenta,
    hilo::id::IdHilo,
    jetstream::admin::JestStreamAdminConexion,
    registrador::Registrador,
};

use super::{conexion::ConexionDeCliente, hilo::Hilo};

type InfoHilo = (
    Sender<(IdConexion, Box<dyn Conexion + Send>)>,
    JoinHandle<()>,
);

pub struct Servidor {
    pub configuracion: Configuracion,
    hilos: Vec<InfoHilo>,
    proximo_id_hilo: usize, // Cada conexión que se genera hay que asignarla a un hilo. Con esto determino a que hilo se lo doy. Si ponemos IdHilo no sirve como indice para Vec, pero si se puede convertir usize a IdHilo
    ultimo_id_conexion: IdConexion, // Cada id tiene que ser único por cada conexion. Se incrementa cada vez que se crea una nueva conexion
    registrador: Registrador,
    pub cuentas: Option<Arc<Vec<Cuenta>>>,
}

impl Servidor {
    pub fn desde_configuracion(configuracion: Configuracion) -> Servidor {
        // La cantidad es la cantidad de hilos que se van a crear
        // Vector con los canales para enviar nuevas conexiones y handle de los threads
        let mut hilos = Vec::new();

        // Puntas emisoras de los canales para enviar mensajes a los hilos
        let mut canales_enviar = Vec::new();
        // Puntas receptoras de los canales para recibir mensajes de los hilos
        let mut canales_recibir = Vec::new();

        let no_registrar_info = configuracion.obtener::<bool>("noinfo").unwrap_or(false);

        // `logger`
        let registrador = Registrador::new(Some(no_registrar_info));

        let cantidad = configuracion.obtener::<usize>("hilos").unwrap_or(4);

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
            configuracion,
            proximo_id_hilo: 0,
            ultimo_id_conexion: 0,
            registrador,
            cuentas: None,
        }
    }

    pub fn cargar_cuentas(&mut self, ruta_archivo_cuentas: String) -> io::Result<()> {
        let cuentas = Cuenta::cargar(&ruta_archivo_cuentas)?;
        self.cuentas = Some(Arc::new(cuentas));
        Ok(())
    }

    fn nuevo_id_conexion(&mut self) -> IdConexion {
        self.ultimo_id_conexion += 1;
        self.ultimo_id_conexion
    }

    pub fn iniciar(mut servidor: Servidor) -> JoinHandle<()> {
        thread::spawn(move || {
            servidor.inicio();
        })
    }

    pub fn tls_acceptor(&self) -> io::Result<Option<TlsAcceptor>> {
        let ruta_cert = self.configuracion.obtener::<String>("cert");

        let ruta_key = self.configuracion.obtener::<String>("key");

        if let (Some(c), Some(k)) = (ruta_cert, ruta_key) {
            let cert = std::fs::read(c)?;
            let key = std::fs::read(k)?;

            let identity = Identity::from_pkcs8(&cert, &key)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            let acceptor =
                TlsAcceptor::new(identity).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            return Ok(Some(acceptor));
        }

        Ok(None)
    }

    pub fn direccion(&self) -> String {
        self.configuracion
            .obtener::<String>("direccion")
            .unwrap_or("127.0.0.1".to_string())
    }

    pub fn puerto(&self) -> u16 {
        self.configuracion.obtener::<u16>("puerto").unwrap_or(4222)
    }

    pub fn puerto_tls(&self) -> u16 {
        self.configuracion
            .obtener::<u16>("puerto_tls")
            .unwrap_or(8222)
    }

    pub fn escuchar_sin_tls(&self, tx: Sender<Box<dyn Stream + Send>>) -> io::Result<()> {
        let listener = TcpListener::bind(format!("{}:{}", self.direccion(), self.puerto()))?;

        println!("Escuchando en {}:{}", self.direccion(), self.puerto());

        thread::spawn(move || {
            for conn in listener.incoming() {
                match conn {
                    Ok(stream) => {
                        tx.send(Box::new(stream)).unwrap();
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        });
        Ok(())
    }

    pub fn escuchar_con_tls(&self, tx: Sender<Box<dyn Stream + Send>>) -> io::Result<()> {
        let acceptor = self.tls_acceptor()?;

        if let Some(acceptor) = acceptor {
            let listener =
                TcpListener::bind(format!("{}:{}", self.direccion(), self.puerto_tls()))?;

            println!(
                "Escuchando TLS en {}:{}",
                self.direccion(),
                self.puerto_tls()
            );

            thread::spawn(move || {
                for conn in listener.incoming() {
                    match conn {
                        Ok(stream) => {
                            if let Ok(stream_clone) = stream.try_clone() {
                                match acceptor.accept(stream) {
                                    Ok(stream) => {
                                        if let Ok(()) = stream_clone.set_nonblocking(true) {
                                            tx.send(Box::new(stream)).unwrap();
                                        }
                                    }
                                    Err(e) => eprintln!("Error: {}", e),
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }
                }
            });
        }

        Ok(())
    }

    pub fn inicio(&mut self) {
        let (tx_conexiones, rx_conexiones) = channel::<Box<dyn Conexion + Send>>();

        let id_conexion = self.nuevo_id_conexion();
        let _ = tx_conexiones.send(Box::new(JestStreamAdminConexion::new(
            id_conexion,
            tx_conexiones.clone(),
            self.registrador.clone(),
        )));

        let (tx, rx) = mpsc::channel();

        self.escuchar_sin_tls(tx.clone())
            .expect("No se pudo iniciar el servidor");

        self.escuchar_con_tls(tx)
            .expect("No se pudo iniciar el servidor con tls");

        loop {
            while let Ok(stream) = rx.try_recv() {
                // Creamos una copia del logger para la nueva conexion
                let mut registrador_para_nueva_conexion = self.registrador.clone();
                // Establecemos el hilo actual para la nueva conexion
                registrador_para_nueva_conexion.establecer_hilo(self.proximo_id_hilo as IdHilo);

                // Generamos un nuevo id único para la nueva conexión
                let id_conexion = self.nuevo_id_conexion();

                let conexion = ConexionDeCliente::new(
                    id_conexion,
                    stream,
                    registrador_para_nueva_conexion,
                    self.cuentas.clone(),
                );

                let (tx, _) = &self.hilos[self.proximo_id_hilo];
                match tx.send((id_conexion, Box::new(conexion))) {
                    // Envio la conexion al hilo
                    Ok(_) => {
                        self.proximo_id_hilo = (self.proximo_id_hilo + 1) % self.hilos.len();
                    }
                    Err(e) => {
                        panic!("Error: {}", e);
                    }
                }
            }

            while let Ok(mut conexion) = rx_conexiones.try_recv() {
                let id_conexion = self.nuevo_id_conexion();

                let (tx, _) = &self.hilos[self.proximo_id_hilo];

                conexion.setear_id_conexion(id_conexion);

                match tx.send((conexion.obtener_id(), conexion)) {
                    Ok(_) => {
                        self.proximo_id_hilo = (self.proximo_id_hilo + 1) % self.hilos.len();
                    }
                    Err(e) => {
                        panic!("Error: {}", e);
                    }
                }
            }
        }
    }
}

use std::{
    collections::HashMap,
    io,
    sync::mpsc::{Receiver, Sender},
};

use lib::{
    camara::Camara,
    configuracion::Configuracion,
    deteccion::Deteccion,
    incidente::Incidente,
    jet_stream::{consumer_config::ConsumerConfig, stream_config::StreamConfig},
    serializables::{
        deserializar_vec,
        guardar::{cargar_serializable, guardar_serializable},
        serializar_vec, Serializable,
    },
};
use messaging_client::cliente::{
    jetstream::{js_suscripcion::JSSuscripcion, JetStream},
    suscripcion::Suscripcion,
    Cliente,
};

use crate::{
    estado::Estado,
    hilo_deteccion_camara::HiloDeteccionCamara,
    interfaz::{comando::Comando, interpretar_comando, respuesta::Respuesta},
};

/// Sistema central de camaras
pub struct Sistema {
    pub estado: Estado,
    pub configuracion: Configuracion,
    enviar_respuesta: Sender<Respuesta>,
    recibir_comandos: Receiver<Comando>,
    recibir_deteccion: Receiver<Deteccion>,
    enviar_deteccion: Sender<Deteccion>,
    detener_deteccion: HashMap<u64, Sender<()>>,
}

impl Sistema {
    pub fn new(
        estado: Estado,
        configuracion: Configuracion,
        enviar_respuesta: Sender<Respuesta>,
        recibir_comandos: Receiver<Comando>,
    ) -> Self {
        let (enviar_deteccion, recibir_deteccion) = std::sync::mpsc::channel();

        Self {
            estado,
            configuracion,
            enviar_respuesta,
            recibir_comandos,
            recibir_deteccion,
            enviar_deteccion,
            detener_deteccion: HashMap::new(),
        }
    }

    /// Inicia el bucle infinito del sistema
    ///
    /// Está función se encarga de reintentar la ejecución del sistema en caso de error.
    pub fn iniciar(&mut self) -> io::Result<()> {
        self.cargar_camaras()?;
        //Por cada camara, crearle su carpeta de lectura
        let camaras_clones: Vec<Camara> = self.estado.camaras().into_iter().cloned().collect();

        for camara in camaras_clones {
            self.iniciar_hilo_camara(camara)?;
        }

        loop {
            if let Err(e) = self.inicio() {
                eprintln!("Error en hilo principal: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }

    /// Inicia un hilo de detección de una cámara
    ///
    /// Este se encarga de leer las imágenes de la cámara, detectar si hay un incidente y enviarlo al sistema
    fn iniciar_hilo_camara(&mut self, camara: Camara) -> io::Result<()> {
        let ruta = self
            .configuracion
            .obtener("direccion")
            .unwrap_or("./camaras".to_string());
        let ruta = format!("{}/{}", ruta, camara.id);
        std::fs::create_dir_all(&ruta)?;

        let (enviar_detener, recibir_detener) = std::sync::mpsc::channel();

        let enviar_deteccion_clon = self.enviar_deteccion.clone();

        self.detener_deteccion.insert(camara.id, enviar_detener);

        std::thread::spawn(move || {
            HiloDeteccionCamara::new(camara, ruta, enviar_deteccion_clon, recibir_detener)
                .iniciar();
        });

        Ok(())
    }

    /// Inicia el bucle de eventos del sistema
    ///
    /// Este bucle puede terminar por un error de conexión
    fn inicio(&mut self) -> io::Result<()> {
        // Conectar el cliente al servidor de NATS
        let mut cliente = self.conectar()?;
        let mut jet_stream = JetStream::new(cliente.clone());

        jet_stream.crear_stream(&StreamConfig {
            name: "camaras".to_string(),
            subjects: vec!["comandos.camaras".to_string()],
            ..Default::default()
        })?;

        jet_stream.crear_consumer(
            "camaras",
            ConsumerConfig {
                durable_name: "comandos".to_string(),
                filter_subject: Some("comandos.camaras".to_string()),
                ..Default::default()
            },
        )?;

        // Publicar al servidor de NATS el estado de todas las cámaras
        self.publicar_y_guardar_estado_general(&cliente)?;

        let sub_nuevos_incidentes = cliente.suscribirse("incidentes.*.creado", None)?;
        let sub_incidentes_finalizados = cliente.suscribirse("incidentes.*.finalizado", None)?;
        let mut sub_comandos_remotos = jet_stream.suscribirse("camaras", "comandos")?;
        //cliente.suscribirse("comandos.camaras", None)?;

        let sub_incidentes = cliente.suscribirse("incidentes", None)?;

        self.solicitar_actualizacion_incidentes(&cliente)?;

        loop {
            self.ciclo(
                &cliente,
                &sub_nuevos_incidentes,
                &sub_incidentes_finalizados,
                &mut sub_comandos_remotos,
                &sub_incidentes,
            )?;
        }
    }

    /// Conectar el cliente
    fn conectar(&self) -> io::Result<Cliente> {
        let direccion = self
            .configuracion
            .obtener::<String>("direccion")
            .unwrap_or("127.0.0.1".to_string());
        let puerto = self.configuracion.obtener::<u16>("puerto").unwrap_or(4222);
        println!("Conectando al servidor de NATS en {}:{}", direccion, puerto);

        let user = self.configuracion.obtener::<String>("user");
        let pass = self.configuracion.obtener::<String>("pass");

        if user.is_some() || pass.is_some() {
            Cliente::conectar_user_pass(&format!("{}:{}", direccion, puerto), user, pass)
        } else {
            Cliente::conectar(&format!("{}:{}", direccion, puerto))
        }
    }

    /// Publica el estado general de las cámaras y lo guarda en un archivo
    fn publicar_y_guardar_estado_general(&mut self, cliente: &Cliente) -> io::Result<()> {
        let camaras = self.estado.camaras().into_iter().cloned().collect();
        let bytes = serializar_vec(&camaras);
        self.guardar_camaras()?;
        cliente.publicar("camaras", &bytes, None)
    }

    /// Solicita al servidor de NATS la actualización de los incidentes
    fn solicitar_actualizacion_incidentes(&self, cliente: &Cliente) -> io::Result<()> {
        cliente.publicar("comandos.monitoreo", b"actualizar", None)
    }

    /// Guarda las cámaras en un archivo
    fn guardar_camaras(&self) -> io::Result<()> {
        let ruta_archivo_camaras = self
            .configuracion
            .obtener::<String>("camaras")
            .unwrap_or("camaras.csv".to_string());

        let camaras: Vec<Camara> = self.estado.camaras().into_iter().cloned().collect();
        guardar_serializable(&camaras, &ruta_archivo_camaras)
    }

    /// carga las camaras desde un archivo
    ///
    /// Si el archivo no existe, lo crea
    fn cargar_camaras(&mut self) -> io::Result<()> {
        let ruta_archivo_camaras = self
            .configuracion
            .obtener::<String>("camaras")
            .unwrap_or("camaras.csv".to_string());

        let existe = std::path::Path::new(&ruta_archivo_camaras).exists();

        if !existe {
            std::fs::File::create(&ruta_archivo_camaras)?;
        }

        let mut camaras: Vec<Camara> = cargar_serializable(&ruta_archivo_camaras)?;

        for mut camara in camaras.drain(..) {
            camara.incidentes_primarios.clear();
            camara.incidentes_secundarios.clear();
            self.estado.conectar_camara(camara);
        }

        Ok(())
    }

    /// Ciclo de eventos del sistema
    fn ciclo(
        &mut self,
        cliente: &Cliente,
        sub_nuevos_incidentes: &Suscripcion,
        sub_incidentes_finalizados: &Suscripcion,
        sub_comandos: &mut JSSuscripcion,
        sub_incidentes: &Suscripcion,
    ) -> io::Result<()> {
        self.leer_incidentes(
            cliente,
            sub_nuevos_incidentes,
            sub_incidentes_finalizados,
            sub_incidentes,
        )?;
        self.leer_comandos(cliente)?;
        self.leer_comandos_remotos(cliente, sub_comandos)?;
        self.leer_detecciones(cliente)?;

        std::thread::sleep(std::time::Duration::from_millis(5));

        Ok(())
    }

    /// Lee incidentes desde el servidor de NATS
    /// y los procesa. Cambia el estado del sistema
    fn leer_incidentes(
        &mut self,
        cliente: &Cliente,
        sub_nuevos_incidentes: &Suscripcion,
        sub_incidentes_finalizados: &Suscripcion,
        sub_incidentes: &Suscripcion,
    ) -> io::Result<()> {
        let mut enviar_actualizacion = false;

        while let Some(mensaje) = sub_nuevos_incidentes.intentar_leer()? {
            match Incidente::deserializar(&mensaje.payload) {
                Ok(incidente) => {
                    self.estado.cargar_incidente(incidente);
                    enviar_actualizacion = true;
                }
                Err(_) => eprintln!("Error al deserializar incidente"),
            }
        }

        while let Some(mensaje) = sub_incidentes_finalizados.intentar_leer()? {
            match Incidente::deserializar(&mensaje.payload) {
                Ok(incidente) => {
                    self.estado.finalizar_incidente(incidente.id);
                    enviar_actualizacion = true;
                }
                Err(_) => eprintln!("Error al deserializar incidente"),
            };
        }

        while let Some(mensaje) = sub_incidentes.intentar_leer()? {
            let incidentes: Vec<Incidente> = deserializar_vec(&mensaje.payload).unwrap_or_default();

            self.estado.finalizar_todos_los_incidentes();

            println!("Actualizados {} incidentes", incidentes.len());

            for incidente in incidentes {
                self.estado.cargar_incidente(incidente);
            }

            enviar_actualizacion = true;
        }

        if enviar_actualizacion {
            self.publicar_y_guardar_estado_general(cliente)?;
        }

        Ok(())
    }

    /// Lee comandos desde la interfaz y los procesa
    fn leer_comandos(&mut self, cliente: &Cliente) -> io::Result<()> {
        while let Ok(comando) = self.recibir_comandos.try_recv() {
            self.matchear_comandos(cliente, comando)?;
        }

        Ok(())
    }

    /// Lee las detecciones enviadas por las cámaras
    fn leer_detecciones(&mut self, _cliente: &Cliente) -> io::Result<()> {
        while let Ok(deteccion) = self.recibir_deteccion.try_recv() {
            println!("Detección recibida: {:?}", deteccion);
        }

        Ok(())
    }

    /// Procesa los comandos recibidos desde la interfaz
    fn matchear_comandos(&mut self, cliente: &Cliente, comando: Comando) -> io::Result<()> {
        match comando {
            Comando::Conectar(id, lat, lon, rango) => {
                self.comando_conectar_camara(cliente, id, lat, lon, rango)?
            }
            Comando::ConectarSinId(lat, lon, rango) => {
                let id = self.buscar_id_camara();
                self.comando_conectar_camara(cliente, id, lat, lon, rango)?
            }
            Comando::Desconectar(id) => self.comando_desconectar_camara(cliente, id)?,
            Comando::ListarCamaras => self.comando_listar_camaras()?,
            Comando::ModifciarRango(id, rango) => {
                self.comando_modificar_rango(cliente, id, rango)?
            }
            Comando::ModificarUbicacion(id, lat, lon) => {
                self.comando_modificar_ubicacion(cliente, id, lat, lon)?
            }
            Comando::Camara(id) => self.comando_mostrar_camara(id)?,
            Comando::Ayuda => self.comando_ayuda()?,
            Comando::Actualizar => self.publicar_y_guardar_estado_general(cliente)?,
        }
        Ok(())
    }

    /// Devuelve un ID
    ///
    /// Es para el comando ConectarSinId
    fn buscar_id_camara(&self) -> u64 {
        let mut max_id = 1;
        for camara in self.estado.camaras() {
            if camara.id > max_id {
                max_id = camara.id;
            }
        }
        max_id + 1
    }

    /// Lee comandos remotos desde el servidor de NATS
    ///
    /// Lo que se le pide desde la aplicación de monitoreo.
    fn leer_comandos_remotos(
        &mut self,
        cliente: &Cliente,
        sub_comandos_remotos: &mut JSSuscripcion,
    ) -> io::Result<()> {
        while let Some(mensaje) = sub_comandos_remotos.intentar_leer()? {
            let mensaje_texto = String::from_utf8_lossy(&mensaje.payload);

            if let Some(comando) = interpretar_comando(&mensaje_texto) {
                self.matchear_comandos(cliente, comando)?;
            }

            sub_comandos_remotos.ack(&mensaje)?;
        }

        Ok(())
    }

    /// Conecta una camara, siempre que el ID dado no tenga una camara conectada
    fn comando_conectar_camara(
        &mut self,
        cliente: &Cliente,
        id: u64,
        lat: f64,
        lon: f64,
        rango: f64,
    ) -> io::Result<()> {
        if self.estado.camara(id).is_some() {
            return self.responder(Respuesta::Error(
                "Ya existe una cámara con ese ID".to_string(),
            ));
        }
        let camara = Camara::new(id, lat, lon, rango);
        self.estado.conectar_camara(camara);
        self.publicar_y_guardar_estado_general(cliente)?;
        self.responder_ok()
    }

    /// Desconecta una cámara, siempre que exista
    fn comando_desconectar_camara(&mut self, cliente: &Cliente, id: u64) -> io::Result<()> {
        if self.estado.desconectar_camara(id).is_some() {
            self.publicar_y_guardar_estado_general(cliente)?;
            self.responder_ok()
        } else {
            self.responder(Respuesta::Error(
                "No existe una cámara con ese ID".to_string(),
            ))
        }
    }

    /// Devuelve una lista con todas las cámaras conectadas
    fn comando_listar_camaras(&mut self) -> io::Result<()> {
        let camaras: Vec<Camara> = self.estado.camaras().into_iter().cloned().collect();
        if camaras.is_empty() {
            self.responder(Respuesta::Error("No hay cámaras conectadas".to_string()))
        } else {
            self.responder(Respuesta::Camaras(camaras))
        }
    }

    /// Modifica el rango de una cámara, siempre que exista
    fn comando_modificar_rango(
        &mut self,
        cliente: &Cliente,
        id: u64,
        rango: f64,
    ) -> io::Result<()> {
        if self.estado.camara(id).is_none() {
            return self.responder(Respuesta::Error(
                "No existe una cámara con ese ID".to_string(),
            ));
        }

        self.estado.modificar_rango_camara(id, rango);
        self.publicar_y_guardar_estado_general(cliente)?;
        self.responder_ok()
    }

    /// Modifica la ubicación de una cámara, siempre que exista
    fn comando_modificar_ubicacion(
        &mut self,
        cliente: &Cliente,
        id: u64,
        lat: f64,
        lon: f64,
    ) -> io::Result<()> {
        if self.estado.camara(id).is_none() {
            return self.responder(Respuesta::Error(
                "No existe una cámara con ese ID".to_string(),
            ));
        }

        self.estado.modificar_ubicacion_camara(id, lat, lon);
        self.publicar_y_guardar_estado_general(cliente)?;
        self.responder_ok()
    }

    /// Devuelve una cámara según su ID, si es que existe
    fn comando_mostrar_camara(&mut self, id: u64) -> io::Result<()> {
        if let Some(camara) = self.estado.camara(id) {
            self.responder(Respuesta::Camara(camara.clone()))
        } else {
            self.responder(Respuesta::Error(
                "No existe una cámara con ese ID".to_string(),
            ))
        }
    }

    /// Llama a la respuesta ayuda
    fn comando_ayuda(&mut self) -> io::Result<()> {
        self.responder(Respuesta::Ayuda)
    }

    /// Llama a la respuesta Ok
    fn responder_ok(&self) -> io::Result<()> {
        self.responder(Respuesta::Ok)
    }

    /// Envia una respuesta al hilo de la interfaz
    fn responder(&self, respuesta: Respuesta) -> io::Result<()> {
        self.enviar_respuesta
            .send(respuesta)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

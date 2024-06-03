use std::{
    io,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use lib::{
    camara::Camara,
    configuracion::Configuracion,
    incidente::Incidente,
    serializables::{
        deserializar_vec,
        guardar::{cargar_serializable, guardar_serializable},
        serializar_vec, Serializable,
    },
};
use messaging_client::cliente::{suscripcion::Suscripcion, Cliente};

use self::{comando::Comando, estado::Estado};

pub mod comando;
pub mod estado;

/// Sistema de monitoreo.
pub struct Sistema {
    pub estado: Estado,
    pub configuracion: Configuracion,
    recibir_comando: Receiver<Comando>,
    enviar_estado: Sender<Estado>,
    proximo_id_incidente: u64,
}

/// Crea un nuevo sistema e intenta iniciarlo.
pub fn intentar_iniciar_sistema(
    recibir_comando: Receiver<Comando>,
    enviar_estado: Sender<Estado>,
) -> io::Result<()> {
    let estado = Estado::new();

    let configuracion = Configuracion::desde_argv()?;
    let mut sistema = Sistema::new(estado, configuracion, recibir_comando, enviar_estado);

    sistema.iniciar()?;

    Ok(())
}

impl Sistema {
    pub fn new(
        estado: Estado,
        configuracion: Configuracion,
        recibir_comando: Receiver<Comando>,
        enviar_estado: Sender<Estado>,
    ) -> Self {
        Self {
            estado,
            configuracion,
            recibir_comando,
            enviar_estado,
            proximo_id_incidente: 0,
        }
    }

    /// Inicia el bucle infinito del sistema
    ///
    /// Está función se encarga de reintentar la ejecución del sistema en caso de error.
    pub fn iniciar(&mut self) -> io::Result<()> {
        self.cargar_incidentes()?;

        loop {
            if !self.estado.conectado {
                if let Ok(Comando::Configurar(config)) = self.recibir_comando.try_recv() {
                    self.configuracion = config;

                    if let Err(e) = self.inicio() {
                        eprintln!("Error al conectar al sistema: {}", e);
                        self.estado.mensaje_error = Some(format!("{}", e));
                        let _ = self.actualizar_estado_ui();
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                }

                continue;
            }

            if let Err(e) = self.inicio() {
                eprintln!("Error en hilo principal: {}", e);
                self.estado.mensaje_error = Some(format!("{}", e));
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }

    /// Inicia el bucle de eventos del sistema
    ///
    /// Este bucle puede terminar por un error de conexión
    fn inicio(&mut self) -> io::Result<()> {
        // Conectar el cliente al servidor de NATS
        let mut cliente = self.conectar()?;

        let sub_conectado = cliente.suscribirse("comandos.monitoreo.conectado", None)?;

        cliente.publicar("comandos.monitoreo.conectado", b"", None)?;

        if sub_conectado
            .leer_con_limite_de_tiempo(Duration::from_secs(5))?
            .is_some()
        {
            self.estado.conectado = true;
            self.estado.mensaje_error = None;
            drop(sub_conectado);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No se pudo conectar al sistema".to_string(),
            ));
        }

        // Publicar al servidor de NATS el estado de todas las cámaras
        self.publicar_y_guardar_estado_general(&cliente)?;

        let suscripcion_camaras = cliente.suscribirse("camaras", None)?;

        let suscripcion_comandos = cliente.suscribirse("comandos.monitoreo", None)?;

        self.actualizar_estado_ui()?;

        self.solicitar_actualizacion_camaras(&cliente)?;

        loop {
            self.ciclo(&cliente, &suscripcion_camaras, &suscripcion_comandos)?;
        }
    }

    /// Conectar el cliente con usuario y contraeña.
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

    /// Publica el estado general del sistema y lo guarda en un archivo
    fn publicar_y_guardar_estado_general(&mut self, cliente: &Cliente) -> io::Result<()> {
        let incidentes = self.estado.incidentes();
        let bytes = serializar_vec(&incidentes);
        self.guardar_incidentes()?;
        cliente.publicar("incidentes", &bytes, None)
    }

    /// Guarda los incidente serializados en un csv.
    fn guardar_incidentes(&self) -> io::Result<()> {
        let ruta_archivo_incidentes = self
            .configuracion
            .obtener::<String>("incidentes")
            .unwrap_or("incidentes.csv".to_string());

        let incidentes: Vec<Incidente> = self.estado.incidentes();
        guardar_serializable(&incidentes, &ruta_archivo_incidentes)
    }

    /// Carga los incidentes al inicializarse desde un csv.
    ///
    /// Si no existe, lo crea y no se vera ningún incidente al iniciar.
    fn cargar_incidentes(&mut self) -> io::Result<()> {
        let ruta_archivo_incidentes = self
            .configuracion
            .obtener::<String>("incidentes")
            .unwrap_or("incidentes.csv".to_string());

        let existe = std::path::Path::new(&ruta_archivo_incidentes).exists();

        if !existe {
            std::fs::File::create(&ruta_archivo_incidentes)?;
        }

        let mut incidentes: Vec<Incidente> = cargar_serializable(&ruta_archivo_incidentes)?;

        let mut id_max = 1;

        for incidente in incidentes.drain(..) {
            if incidente.id > id_max {
                id_max = incidente.id;
            }

            self.estado.cargar_incidente(incidente);
        }

        self.proximo_id_incidente = id_max + 1;

        Ok(())
    }

    /// Ciclo de eventos del sistema
    fn ciclo(
        &mut self,
        cliente: &Cliente,
        suscripcion_camaras: &Suscripcion,
        suscripcion_comandos: &Suscripcion,
    ) -> io::Result<()> {
        self.leer_camaras(cliente, suscripcion_camaras)?;
        self.leer_comandos(cliente)?;
        self.leer_comandos_remotos(cliente, suscripcion_comandos)?;

        std::thread::sleep(std::time::Duration::from_millis(5));

        Ok(())
    }

    /// Lee cámaras desde el servidor de NATS
    /// y los procesa. Cambia el estado del sistema
    fn leer_camaras(
        &mut self,
        _cliente: &Cliente,
        suscripcion_camaras: &Suscripcion,
    ) -> io::Result<()> {
        if let Some(mensaje) = suscripcion_camaras.intentar_leer()? {
            let camaras: Vec<Camara> = deserializar_vec(&mensaje.payload).unwrap_or_default();

            self.estado.limpiar_camaras();
            for camara in camaras {
                self.estado.conectar_camara(camara);
            }

            self.actualizar_estado_ui()?;
        }

        Ok(())
    }

    /// Lee comandos desde la interfaz y los procesa
    fn leer_comandos(&mut self, cliente: &Cliente) -> io::Result<()> {
        while let Ok(comando) = self.recibir_comando.try_recv() {
            match comando {
                Comando::NuevoIncidente(mut incidente) => {
                    incidente.id = self.proximo_id_incidente;

                    self.proximo_id_incidente += 1;

                    self.estado.cargar_incidente(incidente.clone());
                    self.guardar_incidentes()?;
                    self.publicar_nuevo_incidente(cliente, &incidente)?;
                    self.actualizar_estado_ui()?;
                }
                Comando::ModificarIncidente(incidente) => {
                    self.estado.cargar_incidente(incidente.clone());
                    self.guardar_incidentes()?;
                    self.publicar_nuevo_incidente(cliente, &incidente)?;
                    self.actualizar_estado_ui()?;
                }
                Comando::IncidenteFinalizado(id) => {
                    if let Some(incidente) = self.estado.finalizar_incidente(&id) {
                        self.guardar_incidentes()?;
                        self.publicar_incidente_finalizado(cliente, &incidente)?;
                        self.actualizar_estado_ui()?;
                    }
                }
                Comando::CamaraNuevaUbicacion(id, lat, lon) => {
                    if self.estado.camara(id).is_some() {
                        cliente.publicar(
                            "comandos.camaras",
                            format!("modificar ubicacion {} {} {}", id, lat, lon).as_bytes(),
                            None,
                        )?;
                    }
                }
                Comando::Desconectar => {
                    self.estado.conectado = false;
                    self.configuracion = Configuracion::default();
                    self.actualizar_estado_ui()?;
                    return Err(io::Error::new(io::ErrorKind::Other, "".to_string()));
                }
                Comando::CamaraNuevoRango(id, rango) => {
                    if let Some(_camara) = self.estado.camara(id) {
                        cliente.publicar(
                            "comandos.camaras",
                            format!("modificar rango {} {}", id, rango).as_bytes(),
                            None,
                        )?;
                    }
                }
                Comando::ConectarCamara(lat, lon, rango) => {
                    cliente.publicar(
                        "comandos.camaras",
                        format!("conectar {} {} {}", lat, lon, rango).as_bytes(),
                        None,
                    )?;
                    //self.estado.conectar_camara(camara);
                    self.actualizar_estado_ui()?;
                }
                Comando::DesconectarCamara(id) => {
                    if let Some(_camara) = self.estado.camara(id) {
                        cliente.publicar(
                            "comandos.camaras",
                            format!("desconectar {}", id).as_bytes(),
                            None,
                        )?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Lee comandos remotos desde el servidor de NATS y los procesa.
    fn leer_comandos_remotos(
        &mut self,
        cliente: &Cliente,
        suscripcion_comandos: &Suscripcion,
    ) -> io::Result<()> {
        if let Some(mensaje) = suscripcion_comandos.intentar_leer()? {
            let comando = String::from_utf8_lossy(&mensaje.payload);

            if comando.eq("actualizar") {
                self.publicar_y_guardar_estado_general(cliente)?;
            } else {
                println!("Comando desconocido: {}", comando);
            }
        }

        Ok(())
    }

    /// Publica un nuevo incidente en el servidor de NATS.
    fn publicar_nuevo_incidente(&self, cliente: &Cliente, incidente: &Incidente) -> io::Result<()> {
        let bytes = incidente.serializar();
        let topico = format!("incidentes.{}.creado", incidente.id);
        cliente.publicar(&topico, &bytes, None)
    }

    /// Publica un incidente finalizado en el servidor de NATS.
    fn publicar_incidente_finalizado(
        &self,
        cliente: &Cliente,
        incidente: &Incidente,
    ) -> io::Result<()> {
        let bytes = incidente.serializar();
        let topico = format!("incidentes.{}.finalizado", incidente.id);
        cliente.publicar(&topico, &bytes, None)
    }

    /// Actualiza el estado de la interfaz de usuario
    fn actualizar_estado_ui(&self) -> io::Result<()> {
        self.enviar_estado.send(self.estado.clone()).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Error al enviar estado a la interfaz: {}", e),
            )
        })
    }

    /// Solicita la actualización de las cámaras al servidor de NATS.
    fn solicitar_actualizacion_camaras(&self, cliente: &Cliente) -> io::Result<()> {
        cliente.publicar("comandos.camaras", b"actualizar", None)
    }
}

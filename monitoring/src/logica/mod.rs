use std::{
    collections::HashSet,
    io,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use lib::{
    camara::Camara,
    configuracion::Configuracion,
    deteccion::Deteccion,
    dron::Dron,
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
    ultimo_ciclo: i64,
    actualizar_estado: bool,
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
            ultimo_ciclo: 0,
            actualizar_estado: false,
        }
    }

    /// Inicia el bucle infinito del sistema
    ///
    /// Está función se encarga de reintentar la ejecución del sistema en caso de error.
    pub fn iniciar(&mut self) -> io::Result<()> {
        self.cargar_incidentes()?;

        loop {
            self.actualizar_estado_ui()?;

            if !self.estado.conectado {
                if let Ok(Comando::Configurar(config)) = self.recibir_comando.try_recv() {
                    self.configuracion = config;

                    if let Err(e) = self.inicio() {
                        eprintln!("Error al conectar al sistema: {}", e);
                        self.estado.mensaje_error = Some(format!("{}", e));
                        self.actualizar_estado_ui()?;
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }

            if let Err(e) = self.inicio() {
                eprintln!("Error en hilo principal: {}", e);
                self.estado.mensaje_error = Some(format!("{}", e));
                std::thread::sleep(std::time::Duration::from_secs(1));
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
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

        let suscripcion_estado_drone = cliente.suscribirse("drones.*", None)?;

        let suscripcion_detecciones = cliente.suscribirse("incidentes.deteccion", None)?;

        self.requerir_actualizar_estado_ui();

        self.solicitar_actualizacion_camaras(&cliente)?;

        loop {
            self.ciclo(
                &cliente,
                &suscripcion_camaras,
                &suscripcion_comandos,
                &suscripcion_estado_drone,
                &suscripcion_detecciones,
            )?;

            self.ciclo_cada_un_segundo(
                &cliente,
                &suscripcion_camaras,
                &suscripcion_comandos,
                &suscripcion_estado_drone,
            )?;

            if self.actualizar_estado {
                self.actualizar_estado_ui()?;
            }
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
        let usar_tls = self.configuracion.obtener::<bool>("tls").unwrap_or(false);

        if user.is_some() || pass.is_some() {
            Cliente::conectar_user_pass_tls(
                &format!("{}:{}", direccion, puerto),
                user,
                pass,
                usar_tls,
            )
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
        suscripcion_estado_drone: &Suscripcion,
        suscripcion_detecciones: &Suscripcion,
    ) -> io::Result<()> {
        self.leer_camaras(cliente, suscripcion_camaras)?;
        self.leer_comandos(cliente)?;
        self.leer_comandos_remotos(cliente, suscripcion_comandos)?;
        self.leer_estado_drones(cliente, suscripcion_estado_drone)?;
        self.leer_detecciones(cliente, suscripcion_detecciones)?;

        std::thread::sleep(std::time::Duration::from_millis(30));

        Ok(())
    }

    /// El tiempo máximo que dura un incidente atendido
    fn limite_de_tiempo_incidente_atendidio(&self) -> i64 {
        self.configuracion
            .obtener::<i64>("limite_tiempo_incidente_atendido")
            .unwrap_or(300)
            * 1000
    }

    /// El tiempo máximo que puede durar un incidente hasta que venza (sea atendido o no)
    fn limite_de_tiempo_incidente_total(&self) -> u64 {
        self.configuracion
            .obtener::<u64>("limite_tiempo_incidente_total")
            .unwrap_or(1200)
            * 1000
    }

    /// La cantidad de drones que necesita un incidente para ser atendido
    fn cantidad_de_drones_por_incidente(&self) -> usize {
        self.configuracion
            .obtener::<usize>("drones_por_incidente")
            .unwrap_or(2)
    }

    /// Ciclo que se ejecuta cada segundo
    ///
    /// Se encarga de finalizar los incidentes que cumplieron su tiempo de vida
    fn ciclo_cada_un_segundo(
        &mut self,
        cliente: &Cliente,
        _suscripcion_camaras: &Suscripcion,
        _suscripcion_comandos: &Suscripcion,
        _suscripcion_estado_drone: &Suscripcion,
    ) -> io::Result<()> {
        let ahora = chrono::offset::Local::now().timestamp_millis();

        if self.ultimo_ciclo + 1000 < ahora {
            self.ultimo_ciclo = ahora;
        } else {
            return Ok(());
        }

        for mut incidente in self.estado.incidentes() {
            if incidente.inicio + self.limite_de_tiempo_incidente_total() < ahora as u64 {
                if let Some(incidente) = self.estado.finalizar_incidente(&incidente.id) {
                    self.guardar_incidentes()?;
                    self.publicar_incidente_finalizado(cliente, &incidente)?;
                    self.requerir_actualizar_estado_ui();
                }
                continue;
            }

            // Si por alguna razon hay mas de 2 (o el valor configurado) drones, saco uno.
            let drones_del_incidente = self.estado.drones_incidente(&incidente.id);
            if drones_del_incidente.len() > self.cantidad_de_drones_por_incidente() {
                if let Some(ultimo_dron) = drones_del_incidente.last() {
                    self.desasignar_incidente_al_dron(cliente, incidente.id, ultimo_dron)?;
                }
            }

            // Drones asignados al incidente
            let drones_incidente = self.estado.drones_incidente(&incidente.id);

            // Drones que están efectivamente cerca del incidente
            let mut drones_cerca = 0;
            for dron in drones_incidente.iter() {
                if dron.posicion.distancia(&incidente.posicion()) < 50. {
                    drones_cerca += 1;
                }
            }

            // Solo contamos el tiempo no solo si los drones etán asignados pero si están cerca
            if drones_cerca >= self.cantidad_de_drones_por_incidente() {
                incidente.tiempo_atendido += 1000;
                self.estado.cargar_incidente(incidente.clone());
            }
        }

        for incidente in self.estado.incidentes() {
            // Si el incidente ya fue atendido, lo finalizo
            if incidente.tiempo_atendido > self.limite_de_tiempo_incidente_atendidio() {
                if let Some(incidente) = self.estado.finalizar_incidente(&incidente.id) {
                    self.guardar_incidentes()?;
                    self.publicar_incidente_finalizado(cliente, &incidente)?;
                    self.requerir_actualizar_estado_ui();
                }
            }
        }

        // Eliminar drones que no aparecen hace más de 10 segundos
        self.estado.limpiar_drones();

        self.asignar_incidentes_sin_asignar(cliente)?;

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

            self.requerir_actualizar_estado_ui();
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
                    self.requerir_actualizar_estado_ui();
                    self.asignar_incidentes_sin_asignar(cliente)?;
                }
                Comando::ModificarIncidente(incidente) => {
                    self.estado.cargar_incidente(incidente.clone());
                    self.guardar_incidentes()?;
                    self.publicar_nuevo_incidente(cliente, &incidente)?;
                    self.requerir_actualizar_estado_ui();
                    self.asignar_incidentes_sin_asignar(cliente)?;
                }
                Comando::ModificarUbicacionIncidente(incidente) => {
                    self.estado.cargar_incidente(incidente.clone());
                    self.guardar_incidentes()?;
                    for dron in self.estado.drones_incidente(&incidente.id) {
                        self.desasignar_incidente_al_dron(cliente, incidente.id, dron)?;
                    }
                    self.publicar_nuevo_incidente(cliente, &incidente)?;
                    self.requerir_actualizar_estado_ui();
                    self.asignar_incidentes_sin_asignar(cliente)?;
                }
                Comando::IncidenteFinalizado(id) => {
                    if let Some(incidente) = self.estado.finalizar_incidente(&id) {
                        self.guardar_incidentes()?;
                        self.publicar_incidente_finalizado(cliente, &incidente)?;
                        self.requerir_actualizar_estado_ui();
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
                    self.requerir_actualizar_estado_ui();
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
                    self.requerir_actualizar_estado_ui();
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

    /// Lee el estado de los drones desde el servidor de NATS y los procesa.
    fn leer_estado_drones(
        &mut self,
        _cliente: &Cliente,
        suscripcion_estado_drone: &Suscripcion,
    ) -> io::Result<()> {
        if let Some(mensaje) = suscripcion_estado_drone.intentar_leer()? {
            if let Ok(drone) = Dron::deserializar(&mensaje.payload) {
                self.estado.cargar_dron(drone);
                self.requerir_actualizar_estado_ui();
            }
        }

        Ok(())
    }

    /// Lee las detecciones desde el sistema de cámaras
    fn leer_detecciones(
        &mut self,
        cliente: &Cliente,
        suscripcion_detecciones: &Suscripcion,
    ) -> io::Result<()> {
        if let Some(mensaje) = suscripcion_detecciones.intentar_leer()? {
            if let Ok(deteccion) = Deteccion::deserializar(&mensaje.payload) {
                let mut incidente = Incidente::new(
                    0,
                    format!("{}", deteccion.detalle()),
                    deteccion.posicion.lat,
                    deteccion.posicion.lon,
                    chrono::offset::Local::now().timestamp_millis() as u64,
                );

                incidente.id = self.proximo_id_incidente;
                self.proximo_id_incidente += 1;

                incidente.deteccion = Some(deteccion);

                self.estado.cargar_incidente(incidente.clone());
                self.guardar_incidentes()?;
                self.publicar_nuevo_incidente(cliente, &incidente)?;
                self.requerir_actualizar_estado_ui();
                self.asignar_incidentes_sin_asignar(cliente)?;
            }
        }

        Ok(())
    }

    /// En base a los incidentes conocidos sin asignar y todos los drones conocidos,
    /// Busca asignar a cada incidente los dron más cercanos que necesite.
    fn asignar_incidentes_sin_asignar(&mut self, cliente: &Cliente) -> io::Result<()> {
        // Drones disponibles que no están atendiendo un incidente
        let mut drones = self.estado.drones_disponibles();

        // De los disponibles, lo que ya usé en esta llamada de la función
        let mut drones_usados = HashSet::new();

        // Recorrer por cada incidente sin asignar al 100%
        // Incidente, cuantos drones faltan por ser asignados a ese incidente
        for (incidente, drones_restantes) in self
            .estado
            .incidentes_sin_asignar(self.cantidad_de_drones_por_incidente())
        {
            // Si no hay drone F
            if drones.is_empty() {
                break;
            }

            // Ordenar los drones por distancia al incidente
            drones.sort_by(|a, b| {
                let distancia_a = a.posicion.distancia(&incidente.posicion());
                let distancia_b = b.posicion.distancia(&incidente.posicion());

                distancia_a.partial_cmp(&distancia_b).unwrap()
            });

            // Cuantos drones ya asigné a este incidente
            let mut asignados = 0;

            for dron in drones.iter() {
                // Si ya usé este dron, no lo puedo volver a usar
                if drones_usados.contains(&dron.id) {
                    continue;
                }

                // Si ya asigné todos los drones que necesito, salgo del ciclo
                if asignados == drones_restantes {
                    break;
                }

                if !dron.incidente_en_rango(incidente) {
                    continue;
                }

                asignados += 1;

                self.asignar_incidente_a_dron(cliente, incidente.id, dron)?;
                drones_usados.insert(dron.id);
            }
        }

        Ok(())
    }

    /// Asigna un incidente a un dron
    ///
    /// envia un mensaje de NATS al dron para que atienda el incidente
    fn asignar_incidente_a_dron(
        &self,
        cliente: &Cliente,
        id_incidente: u64,
        drone: &Dron,
    ) -> io::Result<()> {
        if let Some(incidente) = self.estado.incidente(id_incidente) {
            cliente.publicar(
                &format!("drones.{}.comandos", drone.id),
                format!("atender_incidente {}", incidente.serializar_string()).as_bytes(),
                None,
            )?;
        }

        Ok(())
    }

    /// Desasigna un incidente a un dron
    ///
    /// envia un mensaje de NATS al dron para que desatienda el incidente
    fn desasignar_incidente_al_dron(
        &self,
        cliente: &Cliente,
        id_incidente: u64,
        drone: &Dron,
    ) -> io::Result<()> {
        if let Some(incidente) = self.estado.incidente(id_incidente) {
            cliente.publicar(
                &format!("drones.{}.comandos", drone.id),
                format!("desasignar_incidente {}", incidente.serializar_string()).as_bytes(),
                None,
            )?;
        }

        Ok(())
    }

    /// Publica un nuevo incidente en el servidor de NATS.
    fn publicar_nuevo_incidente(&self, cliente: &Cliente, incidente: &Incidente) -> io::Result<()> {
        let bytes = incidente.serializar();
        let topico = format!("incidentes.{}.creado", incidente.id);
        cliente.publicar(&topico, &bytes, None)?;

        Ok(())
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

    /// Marca que se debe actualizar la ui
    fn requerir_actualizar_estado_ui(&mut self) {
        self.actualizar_estado = true;
    }

    /// Actualiza el estado de la interfaz de usuario
    fn actualizar_estado_ui(&mut self) -> io::Result<()> {
        self.enviar_estado.send(self.estado.clone()).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Error al enviar estado a la interfaz: {}", e),
            )
        })?;

        self.actualizar_estado = false;
        Ok(())
    }

    /// Solicita la actualización de las cámaras al servidor de NATS.
    fn solicitar_actualizacion_camaras(&self, cliente: &Cliente) -> io::Result<()> {
        cliente.publicar("comandos.camaras", b"actualizar", None)
    }
}

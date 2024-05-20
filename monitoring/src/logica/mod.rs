use std::{
    io,
    sync::mpsc::{Receiver, Sender},
};

use lib::{
    camara::Camara,
    configuracion::ArchivoConfiguracion,
    incidente::Incidente,
    serializables::{
        deserializar_vec,
        guardar::{cargar_serializable, guardar_serializable},
        serializar_vec, 
    },
};
use messaging_client::cliente::{suscripcion::Suscripcion, Cliente};

use self::{comando::Comando, estado::Estado};

pub mod comando;
pub mod estado;

pub struct Sistema {
    pub estado: Estado,
    pub configuracion: ArchivoConfiguracion,
    recibir_comando: Receiver<Comando>,
    enviar_estado: Sender<Estado>,
}

pub fn intentar_iniciar_sistema(
    recibir_comando: Receiver<Comando>,
    enviar_estado: Sender<Estado>,
) -> io::Result<()> {
    let estado = Estado::new();

    let configuracion = ArchivoConfiguracion::desde_argv()?;
    let mut sistema = Sistema::new(estado, configuracion, recibir_comando, enviar_estado);

    sistema.iniciar()?;

    Ok(())
}

impl Sistema {
    pub fn new(
        estado: Estado,
        configuracion: ArchivoConfiguracion,
        recibir_comando: Receiver<Comando>,
        enviar_estado: Sender<Estado>,
    ) -> Self {
        Self {
            estado,
            configuracion,
            recibir_comando,
            enviar_estado,
        }
    }

    /// Inicia el bucle infinito del sistema
    ///
    /// Está función se encarga de reintentar la ejecución del sistema en caso de error.
    pub fn iniciar(&mut self) -> io::Result<()> {
        self.cargar_incidentes()?;

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
        let mut cliente = self.conectar()?;

        // Publicar al servidor de NATS el estado de todas las cámaras
        self.publicar_y_guardar_estado_general(&cliente)?;

        let suscripcion_camaras = cliente.suscribirse("camaras", None)?;

        loop {
            self.ciclo(&cliente, &suscripcion_camaras)?;
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

    fn publicar_y_guardar_estado_general(&mut self, cliente: &Cliente) -> io::Result<()> {
        let incidentes = self.estado.incidentes().into_iter().cloned().collect();
        let bytes = serializar_vec(&incidentes);
        self.guardar_incidentes()?;
        cliente.publicar("camaras", &bytes, None)
    }

    fn guardar_incidentes(&self) -> io::Result<()> {
        let ruta_archivo_incidentes = self
            .configuracion
            .obtener::<String>("camaras")
            .unwrap_or("camaras.csv".to_string());

        let incidentes: Vec<Incidente> = self.estado.incidentes().into_iter().cloned().collect();
        guardar_serializable(&incidentes, &ruta_archivo_incidentes)
    }

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

        for incidente in incidentes.drain(..) {
            self.estado.agregar_incidente(incidente);
        }

        Ok(())
    }

    /// Ciclo de eventos del sistema
    fn ciclo(&mut self, cliente: &Cliente, suscripcion_camaras: &Suscripcion) -> io::Result<()> {
        self.leer_camaras(cliente, suscripcion_camaras)?;
        self.leer_comandos(cliente)?;
        Ok(())
    }

    /// Lee incidentes desde el servidor de NATS
    /// y los procesa. Cambia el estado del sistema
    fn leer_camaras(
        &mut self,
        _cliente: &Cliente,
        suscripcion_camaras: &Suscripcion,
    ) -> io::Result<()> {
        if let Some(mensaje) = suscripcion_camaras.intentar_leer()? {
            let camaras: Vec<Camara> = deserializar_vec(&mensaje.payload).unwrap_or_default();

            for camara in camaras {
                self.estado.agregar_camara(camara);
            }
        }

        Ok(())
    }

    /// Lee comandos desde la interfaz y los procesa
    fn leer_comandos(&mut self, cliente: &Cliente) -> io::Result<()> {
        while let Ok(comando) = self.recibir_comando.try_recv() {
            match comando {
                Comando::NuevoIncidente(incidente) => {
                    self.estado.agregar_incidente(incidente);
                    self.guardar_incidentes()?;
                    self.publicar_y_guardar_estado_general(cliente)?;
                    self.actualizar_estado_ui()?;
                }
            }
        }

        Ok(())
    }

    fn actualizar_estado_ui(&self) -> io::Result<()> {
        self.enviar_estado.send(self.estado.clone()).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Error al enviar estado a la interfaz: {}", e),
            )
        })
    }
}

// pub fn iniciar_hilo_logica(recibir_comando: Receiver<Comando>, enviar_estado: Sender<Estado>) {
//     thread::spawn(move || inicio(recibir_comando, enviar_estado));
// }

// pub fn inicio(recibir_comando: Receiver<Comando>, enviar_estado: Sender<Estado>) {
//     let mut estado = Estado::new();

//     loop {
//         if let Err(e) = inicio_conexion(&mut estado, &recibir_comando, &enviar_estado) {
//             println!("Error: {}", e);
//         }
//     }
// }

// pub fn inicio_conexion(
//     estado: &mut Estado,
//     recibir_comando: &Receiver<Comando>,
//     enviar_estado: &Sender<Estado>,
// ) -> Result<(), String> {
//     loop {
//         if let Ok(comando) = recibir_comando.try_recv() {
//             match comando {
//                 Comando::NuevoIncidente(incidente) => {
//                     estado.agregar_incidente(incidente);
//                     let _ = enviar_estado.send(estado.clone());
//                 }
//             }
//         }
//     }
// }

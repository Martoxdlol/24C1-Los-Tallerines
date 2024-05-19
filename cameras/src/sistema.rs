use std::{
    io,
    sync::mpsc::{Receiver, Sender},
};

use lib::{
    coordenadas::Coordenadas,
    incidente::Incidente,
    serializables::{camara_serializable::CamaraSerializable, serializar_vec, Serializable},
};
use messaging_client::cliente::{suscripcion::Suscripcion, Cliente};

use crate::{
    camara::Camara,
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
        let mut cliente = self.conectar()?;

        // Publicar al servidor de NATS el estado de todas las cámaras
        self.publicar_estado_general(&cliente)?;

        let sub_nuevos_incidentes = cliente.suscribirse("incidentes.*.creado", None)?;
        let sub_incidentes_finalizados = cliente.suscribirse("incidentes.*.finalizado", None)?;

        loop {
            self.ciclo(
                &cliente,
                &sub_nuevos_incidentes,
                &sub_incidentes_finalizados,
            )?;
        }
    }

    /// Conectar el cliente
    fn conectar(&self) -> io::Result<Cliente> {
        Cliente::conectar("127.0.0.1:4222")
    }

    fn publicar_estado_general(&mut self, cliente: &Cliente) -> io::Result<()> {
        let camaras_serializables: Vec<CamaraSerializable> = self
            .estado
            .camaras()
            .into_iter()
            .map(|camara| camara.serializable())
            .collect();
        let bytes = serializar_vec(&camaras_serializables);
        cliente.publicar("camaras", &bytes, None)
    }

    fn ciclo(
        &mut self,
        cliente: &Cliente,
        sub_nuevos_incidentes: &Suscripcion,
        sub_clientes_finalizados: &Suscripcion,
    ) -> io::Result<()> {
        self.leer_incidentes(cliente, sub_nuevos_incidentes, sub_clientes_finalizados)?;
        self.leer_comandos(cliente)?;
        Ok(())
    }

    fn leer_incidentes(
        &mut self,
        _cliente: &Cliente,
        sub_nuevos_incidentes: &Suscripcion,
        sub_clientes_finalizados: &Suscripcion,
    ) -> io::Result<()> {
        if let Some(mensaje) = sub_nuevos_incidentes.intentar_leer()? {
            match Incidente::deserializar(&mensaje.payload) {
                Ok(incidente) => self.estado.nuevo_incidente(incidente),
                Err(_) => eprintln!("Error al deserializar incidente"),
            }
        }

        if let Some(mensaje) = sub_clientes_finalizados.intentar_leer()? {
            match Incidente::deserializar(&mensaje.payload) {
                Ok(incidente) => self.estado.incidente_finalizado(incidente.id),
                Err(_) => eprintln!("Error al deserializar incidente"),
            }
        }

        Ok(())
    }

    fn leer_comandos(&mut self, cliente: &Cliente) -> io::Result<()> {
        while let Ok(comando) = self.recibir_comandos.try_recv() {
            match comando {
                Comando::Conectar(id, lat, lon, rango) => {
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
        let camara = Camara::new(id, lat, lon, rango);

        let camara_serializada = camara.serializable().serializar();

        match self.estado.conectar_camara(camara) {
            Ok(()) => {
                cliente.publicar(
                    &format!("camaras.{}.conectada", id),
                    &camara_serializada,
                    None,
                )?;
                self.enviar_respuesta(Respuesta::Ok)
            }
            Err(mensaje_de_error) => self.enviar_respuesta(Respuesta::Error(mensaje_de_error)),
        }
    }

    fn comando_desconectar_camara(&mut self, cliente: &Cliente, id: u64) -> io::Result<()> {
        match self.estado.desconectar_camara(id) {
            Ok(camara) => {
                cliente.publicar(
                    &format!("camaras.{}.desconectada", id),
                    &camara.serializable().serializar(),
                    None,
                )?;
                self.enviar_respuesta(Respuesta::Ok)
            }
            Err(mensaje_de_error) => self.enviar_respuesta(Respuesta::Error(mensaje_de_error)),
        }
    }

    fn comando_listar_camaras(&mut self) -> io::Result<()> {
        let camaras: Vec<Camara> = self.estado.camaras().into_iter().cloned().collect();
        if camaras.is_empty() {
            self.enviar_respuesta(Respuesta::Error("No hay cámaras conectadas".to_string()))
        } else {
            self.enviar_respuesta(Respuesta::Camaras(camaras))
        }
    }

    fn comando_modificar_rango(
        &mut self,
        cliente: &Cliente,
        id: u64,
        rango: f64,
    ) -> io::Result<()> {
        match self.estado.modificar_rango(id, rango) {
            Ok(()) => {
                cliente.publicar(&format!("camaras.{}.rango_modificado", id), &[], None)?;
                self.enviar_respuesta(Respuesta::Ok)
            }
            Err(mensaje_de_error) => self.enviar_respuesta(Respuesta::Error(mensaje_de_error)),
        }
    }

    fn comando_modificar_ubicacion(
        &mut self,
        cliente: &Cliente,
        id: u64,
        lat: f64,
        lon: f64,
    ) -> io::Result<()> {
        let pos = Coordenadas::from_lat_lon(lat, lon);
        let incidentes = self.estado.incidentes_en_rango_de_camara(id, &pos);

        if let Some(camara) = self.estado.camara_mut(id) {
            camara.lat = lat;
            camara.lon = lon;

            camara.incidentes = incidentes;

            let camara_serializada = camara.serializable().serializar();

            cliente.publicar(
                &format!("camaras.{}.conectada", id),
                &camara_serializada,
                None,
            )?;

            self.enviar_respuesta(Respuesta::Ok)?;
        } else {
            return self.enviar_respuesta(Respuesta::Error(
                "No existe una cámara con ese ID".to_string(),
            ));
        }

        Ok(())
    }

    fn comando_mostrar_camara(&mut self, id: u64) -> io::Result<()> {
        if let Some(camara) = self.estado.camara(id) {
            self.enviar_respuesta(Respuesta::Camara(camara.clone()))
        } else {
            self.enviar_respuesta(Respuesta::Error(
                "No existe una cámara con ese ID".to_string(),
            ))
        }
    }

    fn comando_ayuda(&mut self) -> io::Result<()> {
        self.enviar_respuesta(Respuesta::Ayuda)
    }

    fn enviar_respuesta(&self, respuesta: Respuesta) -> io::Result<()> {
        self.enviar_respuesta
            .send(respuesta)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

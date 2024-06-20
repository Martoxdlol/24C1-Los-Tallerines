use std::sync::mpsc::Sender;

use lib::{configuracion::Configuracion, incidente::Incidente};

/// Comandos que se pueden enviar al hilo de la lógica
pub enum Comando {
    Configurar(Configuracion),
    Desconectar,
    NuevoIncidente(Incidente),
    ModificarIncidente(Incidente),
    ModificarUbicacionIncidente(Incidente),
    IncidenteFinalizado(u64),
    ConectarCamara(f64, f64, f64),
    DesconectarCamara(u64),
    CamaraNuevaUbicacion(u64, f64, f64),
    CamaraNuevoRango(u64, f64),
}

impl Comando {
    /// Envía un comando al hilo de la lógica
    fn enviar(canal: &Sender<Comando>, comando: Self) {
        let _ = canal.send(comando);
    }

    /// Envía un nuevo incidente al hilo de la lógica
    pub fn nuevo_incidente(canal: &Sender<Comando>, incidente: Incidente) {
        Self::enviar(canal, Comando::NuevoIncidente(incidente));
    }

    /// Envía un incidente modificado al hilo de la lógica
    pub fn modificar_incidente(canal: &Sender<Comando>, incidente: Incidente) {
        Self::enviar(canal, Comando::ModificarIncidente(incidente));
    }

    /// Envía un incidente con ubicación modificada al hilo de la lógica
    pub fn modificar_ubicacion_incidente(canal: &Sender<Comando>, incidente: Incidente) {
        Self::enviar(canal, Comando::ModificarUbicacionIncidente(incidente));
    }

    /// Envía un incidente finalizado al hilo de la lógica
    pub fn incidente_finalizado(canal: &Sender<Comando>, id: u64) {
        Self::enviar(canal, Comando::IncidenteFinalizado(id));
    }

    /// Envía una nueva ubicación para una cámara al hilo de la lógica
    pub fn camara_nueva_ubicacion(canal: &Sender<Comando>, id: u64, lat: f64, lon: f64) {
        Self::enviar(canal, Comando::CamaraNuevaUbicacion(id, lat, lon));
    }

    /// Envía un nuevo rango para una cámara al hilo de la lógica
    pub fn camara_nuevo_rango(canal: &Sender<Comando>, id: u64, rango: f64) {
        Self::enviar(canal, Comando::CamaraNuevoRango(id, rango));
    }

    /// Envía un comando para conectar una cámara al hilo de la lógica
    pub fn conectar_camara(canal: &Sender<Comando>, lat: f64, lon: f64, rango: f64) {
        Self::enviar(canal, Comando::ConectarCamara(lat, lon, rango));
    }

    /// Envía un comando para desconectar una cámara al hilo de la lógica
    pub fn desconectar_camara(canal: &Sender<Comando>, id: u64) {
        Self::enviar(canal, Comando::DesconectarCamara(id));
    }

    /// Envía un comando para configurar la lógica
    pub fn configurar(canal: &Sender<Comando>, configuracion: Configuracion) {
        Self::enviar(canal, Comando::Configurar(configuracion));
    }

    /// Envía un comando para desconectar la lógica. Para cerrar la aplicación.
    pub fn desconectar(canal: &Sender<Comando>) {
        Self::enviar(canal, Comando::Desconectar);
    }
}

use std::sync::mpsc::Sender;

use lib::{
    camara::{self, Camara},
    incidente::Incidente,
};
use walkers::Position;

/// Comandos que se pueden enviar al hilo de la lógica
pub enum Comando {
    NuevoIncidente(Incidente),
    ModificarIncidente(Incidente),
    IncidenteFinalizado(u64),
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

    pub fn modificar_incidente(canal: &Sender<Comando>, incidente: Incidente) {
        Self::enviar(canal, Comando::ModificarIncidente(incidente));
    }

    /// Envía un incidente finalizado al hilo de la lógica
    pub fn incidente_finalizado(canal: &Sender<Comando>, id: u64) {
        Self::enviar(canal, Comando::IncidenteFinalizado(id));
    }

    pub fn camara_nueva_ubicacion(canal: &Sender<Comando>, id: u64, lat: f64, lon: f64) {
        Self::enviar(canal, Comando::CamaraNuevaUbicacion(id, lat, lon));
    }

    pub fn camara_nuevo_rango(canal: &Sender<Comando>, id: u64, rango: f64) {
        Self::enviar(canal, Comando::CamaraNuevoRango(id, rango));
    }
}

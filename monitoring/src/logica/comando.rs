use std::sync::mpsc::Sender;

use lib::incidente::Incidente;

/// Comandos que se pueden enviar al hilo de la lógica
pub enum Comando {
    NuevoIncidente(Incidente),
    ModificarIncidente(Incidente),
    IncidenteFinalizado(u64),
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
}

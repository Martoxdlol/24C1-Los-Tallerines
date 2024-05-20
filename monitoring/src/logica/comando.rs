use std::sync::mpsc::Sender;

use lib::incidente::Incidente;

pub enum Comando {
    NuevoIncidente(Incidente),
    IncidenteFinalizado(u64),
}

impl Comando {
    fn enviar(canal: &Sender<Comando>, comando: Self) {
        let _ = canal.send(comando);
    }

    pub fn nuevo_incidente(canal: &Sender<Comando>, incidente: Incidente) {
        Self::enviar(canal, Comando::NuevoIncidente(incidente));
    }

    pub fn incidente_finalizado(canal: &Sender<Comando>, id: u64) {
        Self::enviar(canal, Comando::IncidenteFinalizado(id));
    }
}

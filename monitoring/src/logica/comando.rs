use std::sync::mpsc::Sender;

use crate::incidente::Incidente;

pub enum Comando {
    NuevoIncidente(Incidente),
}

impl Comando {
    fn enviar(canal: &Sender<Comando>, comando: Self) {
        let _ = canal.send(comando);
    }

    pub fn nuevo_incidente(canal: &Sender<Comando>, incidente: Incidente) {
        Self::enviar(canal, Comando::NuevoIncidente(incidente));
    }
}

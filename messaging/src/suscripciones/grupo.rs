use std::collections::HashSet;

use crate::topico::Topico;

use super::{id::IdSuscripcion, suscripcion::Suscripcion};

pub struct Grupo {
    id: IdSuscripcion,
    topico: Topico,
    suscripciones: HashSet<Suscripcion>
}

impl Grupo {
    pub fn new(id: IdSuscripcion, topico: Topico) -> Self {
        Self {
            id,
            topico,
            suscripciones: HashSet::new()
        }
    }

    pub fn id(&self) -> &IdSuscripcion {
        &self.id
    }

    pub fn topico(&self) -> &Topico {
        &self.topico
    }

    pub fn suscribir(&mut self, suscripcion: Suscripcion) {
        self.suscripciones.insert(suscripcion);
    }

    pub fn desuscribir(&mut self, suscripcion: &Suscripcion) {
        self.suscripciones.remove(suscripcion);
    }
}
use crate::{hilo::id::IdHilo, id_cliente::IdCliente, topico::Topico};

use super::id::IdSuscripcion;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Suscripcion {
    id_hilo: IdHilo,
    id_cliente: IdCliente,
    id: IdSuscripcion,
    topico: Topico,
    id_grupo: Option<IdSuscripcion>
}

impl Suscripcion {
    pub fn new(id_hilo: IdHilo, id_cliente: IdCliente, topico: Topico, id: IdSuscripcion, grupo: Option<IdSuscripcion>) -> Self {
        Self {
            id_hilo,
            id_cliente,
            topico,
            id,
            id_grupo: grupo
        }
    }

    pub fn topico(&self) -> &Topico {
        &self.topico
    }

    pub fn id(&self) -> &IdSuscripcion {
        &self.id
    }

    pub fn id_hilo(&self) -> &IdHilo {
        &self.id_hilo
    }

    pub fn id_cliente(&self) -> &IdCliente {
        &self.id_cliente
    }

    pub fn id_grupo(&self) -> Option<&IdSuscripcion> {
        self.id_grupo.as_ref()
    }
}

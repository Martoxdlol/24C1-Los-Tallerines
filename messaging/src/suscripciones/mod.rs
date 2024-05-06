use std::{
    collections::{HashMap, HashSet},
    rc::{Rc, Weak},
};

use crate::{hilo::id::IdHilo, id_cliente::IdCliente, topico::Topico};

use self::{grupo::Grupo, id::IdSuscripcion, suscripcion::Suscripcion};

pub mod grupo;
pub mod id;
pub mod suscripcion;
pub struct Suscripciones {
    suscripciones: HashSet<Rc<Suscripcion>>,
    suscripciones_por_topico: HashMap<Topico, Vec<Weak<Suscripcion>>>,
    suscripciones_por_cliente: HashMap<IdCliente, Vec<Weak<Suscripcion>>>,
    suscripciones_por_hilo: HashMap<IdHilo, Vec<Weak<Suscripcion>>>,
    grupos: HashMap<IdSuscripcion, Grupo>,
}

impl Suscripciones {
    pub fn new() -> Self {
        Self {
            suscripciones: HashSet::new(),
            suscripciones_por_topico: HashMap::new(),
            suscripciones_por_cliente: HashMap::new(),
            suscripciones_por_hilo: HashMap::new(),
            grupos: HashMap::new(),
        }
    }

    pub fn suscribir(&mut self, suscripcion: Suscripcion) {
        if let Some(id_grupo) = suscripcion.id_grupo() {
            self.suscribir_grupo(suscripcion, *id_grupo);
        }

        let suscripcion_rc = Rc::new(suscripcion);
        let suscripcion_weak = Rc::downgrade(&suscripcion_rc);

        // Insertar en suscripciones HashSet
        self.suscripciones.insert(suscripcion_rc.clone());

        // Insertar en suscripciones_por_topico HashMap
        let suscripciones_por_topico = self
            .suscripciones_por_topico
            .entry(suscripcion_rc.topico().clone())
            .or_insert_with(Vec::new);

        suscripciones_por_topico.push(suscripcion_weak.clone());

        // Insertar en suscripciones_por_hilo HashMap
        let suscripciones_por_cliente = self
            .suscripciones_por_cliente
            .entry(*suscripcion_rc.id_cliente())
            .or_insert_with(Vec::new);

        suscripciones_por_cliente.push(suscripcion_weak.clone());

        // Insertar en suscripciones_por_cliente HashMap
        let suscripciones_por_hilo = self
            .suscripciones_por_hilo
            .entry(*suscripcion_rc.id_hilo())
            .or_insert_with(Vec::new);

        suscripciones_por_hilo.push(suscripcion_weak);
    }

    pub fn desuscribir(&mut self, suscripcion: &Suscripcion) {
        let suscripcion_rc = Rc::new(suscripcion.clone());
        let suscripcion_weak = Rc::downgrade(&suscripcion_rc);

        // Eliminar de suscripciones HashSet
        self.suscripciones.remove(&suscripcion_rc);

        // Eliminar de suscripciones_por_topico HashMap
        if let Some(suscripciones_por_topico) = self
            .suscripciones_por_topico
            .get_mut(suscripcion_rc.topico())
        {
            suscripciones_por_topico.retain(|s| !s.ptr_eq(&suscripcion_weak));
        }

        // Eliminar de suscripciones_por_hilo HashMap
        if let Some(suscripciones_por_hilo) = self
            .suscripciones_por_hilo
            .get_mut(suscripcion_rc.id_hilo())
        {
            suscripciones_por_hilo.retain(|s| !s.ptr_eq(&suscripcion_weak));
        }

        // Eliminar de suscripciones_por_cliente HashMap
        if let Some(suscripciones_por_cliente) = self
            .suscripciones_por_cliente
            .get_mut(suscripcion_rc.id_cliente())
        {
            suscripciones_por_cliente.retain(|s| !s.ptr_eq(&suscripcion_weak));
        }
    }

    fn suscribir_grupo(&mut self, suscripcion: Suscripcion, id_grupo: IdSuscripcion) {
        let grupo = self
            .grupos
            .entry(id_grupo)
            .or_insert_with(|| Grupo::new(id_grupo, suscripcion.topico().clone()));

        grupo.suscribir(suscripcion);
    }

    fn desuscribir_grupo(&mut self, suscripcion: &Suscripcion, id_grupo: IdSuscripcion) {
        if let Some(grupo) = self.grupos.get_mut(&id_grupo) {
            grupo.desuscribir(suscripcion);
        }
    }

    pub fn suscripciones(&self) -> &HashSet<Rc<Suscripcion>> {
        &self.suscripciones
    }

    pub fn visitar_suscripciones_por_topico<F>(&self, topico: &Topico, mut f: F)
    where
        F: FnMut(&Rc<Suscripcion>),
    {
        if let Some(suscripciones_por_topico) = self.suscripciones_por_topico.get(topico) {
            for suscripcion in suscripciones_por_topico {
                if let Some(suscripcion) = suscripcion.upgrade() {
                    f(&suscripcion);
                }
            }
        }
    }

    pub fn visitar_suscripciones_por_cliente<F>(&self, id_cliente: &IdCliente, mut f: F)
    where
        F: FnMut(&Rc<Suscripcion>),
    {
        if let Some(suscripciones_por_cliente) = self.suscripciones_por_cliente.get(id_cliente) {
            for suscripcion in suscripciones_por_cliente {
                if let Some(suscripcion) = suscripcion.upgrade() {
                    f(&suscripcion);
                }
            }
        }
    }

    pub fn visitar_suscripciones_por_hilo<F>(&self, id_hilo: &IdHilo, mut f: F)
    where
        F: FnMut(&Rc<Suscripcion>),
    {
        if let Some(suscripciones_por_hilo) = self.suscripciones_por_hilo.get(id_hilo) {
            for suscripcion in suscripciones_por_hilo {
                if let Some(suscripcion) = suscripcion.upgrade() {
                    f(&suscripcion);
                }
            }
        }
    }

    pub fn hilo_tiene_topico(&self, id_hilo: &IdHilo, topico: &Topico) -> bool {
        if let Some(suscripciones_por_hilo) = self.suscripciones_por_hilo.get(id_hilo) {
            suscripciones_por_hilo.iter().any(|s| {
                if let Some(suscripcion) = s.upgrade() {
                    suscripcion.topico() == topico
                } else {
                    false
                }
            })
        } else {
            false
        }
    }
}

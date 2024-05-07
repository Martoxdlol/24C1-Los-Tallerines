use std::collections::{HashMap, HashSet};

use crate::{conexion::id::IdConexion, hilo::id::IdHilo};

use self::{grupo::Grupo, id::IdSuscripcion, suscripcion::Suscripcion};

pub mod grupo;
pub mod id;
pub mod suscripcion;
pub struct Suscripciones {
    suscripciones: HashSet<Suscripcion>,
    grupos: HashMap<IdSuscripcion, Grupo>,
}

impl Suscripciones {
    pub fn new() -> Self {
        Self {
            suscripciones: HashSet::new(),
            grupos: HashMap::new(),
        }
    }

    pub fn suscribir(&mut self, suscripcion: Suscripcion) {
        self.suscripciones.insert(suscripcion.clone());

        if let Some(id_grupo) = suscripcion.id_grupo() {
            self.suscribir_grupo(suscripcion.clone(), id_grupo);
        }
    }

    pub fn desuscribir(&mut self, id_conexion: IdConexion, id_suscripcion: &IdSuscripcion) {
        let mut desuscripciones_grupos = Vec::new();

        self.suscripciones.retain(|suscripcion| {
            if *suscripcion.id_conexion() == id_conexion && suscripcion.id().eq(id_suscripcion) {
                if let Some(id_grupo) = suscripcion.id_grupo() {
                    desuscripciones_grupos.push((suscripcion.clone(), id_grupo.clone()));
                }
                false
            } else {
                true
            }
        });

        for (suscripcion, id_grupo) in desuscripciones_grupos {
            self.desuscribir_grupo(&suscripcion, &id_grupo);
        }
    }

    fn suscribir_grupo(&mut self, suscripcion: Suscripcion, id_grupo: &IdSuscripcion) {
        let grupo = self.grupos.entry(id_grupo.to_owned()).or_insert(Grupo::new(
            id_grupo.to_owned(),
            suscripcion.topico().clone(),
        ));

        grupo.suscribir(suscripcion);
    }

    fn desuscribir_grupo(&mut self, suscripcion: &Suscripcion, id_grupo: &IdSuscripcion) {
        if let Some(grupo) = self.grupos.get_mut(id_grupo) {
            grupo.desuscribir(suscripcion);
        }
    }

    pub fn suscripciones_topico(&self, topico: &str) -> Vec<&Suscripcion> {
        self.suscripciones
            .iter()
            .filter(|suscripcion| suscripcion.topico().test(topico) && !suscripcion.es_grupo())
            .collect()
    }

    pub fn grupos_topico(&self, topico: &str) -> Vec<&Grupo> {
        self.grupos
            .values()
            .filter(|grupo| grupo.topico().test(topico))
            .collect()
    }

    pub fn hilos_suscriptos_topico(&self, topico: &str) -> HashSet<IdHilo> {
        let mut ids_hilos = HashSet::new();

        for suscripcion in self.suscripciones_topico(topico) {
            ids_hilos.insert(*suscripcion.id_hilo());
        }

        ids_hilos
    }
}

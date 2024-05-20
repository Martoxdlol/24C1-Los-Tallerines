use egui::ahash::HashMap;
use lib::{camara::Camara, incidente::Incidente};

#[derive(Clone, Debug)]
pub struct Estado {
    camaras: HashMap<u64, Camara>,
    incidentes: HashMap<u64, Incidente>,
}

impl Default for Estado {
    fn default() -> Self {
        Self::new()
    }
}

impl Estado {
    pub fn new() -> Self {
        Estado {
            camaras: HashMap::default(),
            incidentes: HashMap::default(),
        }
    }

    pub fn cargar_incidente(&mut self, incidente: Incidente) {
        self.incidentes.insert(incidente.id, incidente);
    }

    pub fn finalizar_incidente(&mut self, id: &u64) -> Option<Incidente> {
        self.incidentes.remove(id)
    }

    pub fn conectar_camara(&mut self, camara: Camara) {
        self.camaras.insert(camara.id, camara);
    }

    pub fn desconectar_camara(&mut self, id: &u64) -> Option<Camara> {
        self.camaras.remove(id)
    }

    pub fn incidentes(&self) -> Vec<Incidente> {
        self.incidentes.values().cloned().collect()
    }

    pub fn camaras(&self) -> Vec<Camara> {
        self.camaras.values().cloned().collect()
    }

    pub fn incidente(&self, id: u64) -> Option<Incidente> {
        self.incidentes.get(&id).cloned()
    }

    pub fn camara(&self, id: u64) -> Option<Camara> {
        self.camaras.get(&id).cloned()
    }

    pub fn limpiar_camaras(&mut self) {
        self.camaras.clear();
    }
}

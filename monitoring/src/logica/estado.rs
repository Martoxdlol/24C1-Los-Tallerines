use crate::{camara::Camara, dron::Dron, incidente::Incidente};

#[derive(Clone)]
pub struct Estado {
    drones: Vec<Dron>,
    camaras: Vec<Camara>,
    incidentes: Vec<Incidente>,
}

impl Default for Estado {
    fn default() -> Self {
        Self::new()
    }
}

impl Estado {
    pub fn new() -> Self {
        Estado {
            drones: vec![],
            camaras: vec![],
            incidentes: vec![],
        }
    }
    pub fn agregar_dron(&mut self, dron: Dron) {
        self.drones.push(dron);
    }
    pub fn agregar_camara(&mut self, camara: Camara) {
        self.camaras.push(camara);
    }
    pub fn agregar_incidente(&mut self, incidente: Incidente) {
        self.incidentes.push(incidente);
    }
    pub fn drones(&self) -> &Vec<Dron> {
        &self.drones
    }
    pub fn camaras(&self) -> &Vec<Camara> {
        &self.camaras
    }
    pub fn incidentes(&self) -> &Vec<Incidente> {
        &self.incidentes
    }
    pub fn drones_mut(&mut self) -> &mut Vec<Dron> {
        &mut self.drones
    }
    pub fn camaras_mut(&mut self) -> &mut Vec<Camara> {
        &mut self.camaras
    }
    pub fn incidentes_mut(&mut self) -> &mut Vec<Incidente> {
        &mut self.incidentes
    }
}

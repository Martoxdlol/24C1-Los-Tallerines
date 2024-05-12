use std::collections::HashMap;

use crate::camara::{id::IdCamara, Camara};

pub struct Estado {
    camaras: HashMap<IdCamara, Camara>,
}

impl Estado {
    pub fn new() -> Self {
        Estado {
            camaras: HashMap::new(),
        }
    }

    pub fn conectar_camara(&mut self, camara: Camara) -> Result<(), String> {
        if self.camaras.contains_key(&camara.id) {
            return Err("Ya existe una cámara con ese ID".to_string());
        }
        self.camaras.insert(camara.id, camara);
        Ok(())
    }

    pub fn desconectar_camara(&mut self, id: IdCamara) -> Result<(), String> {
        if !self.camaras.contains_key(&id) {
            return Err("No existe una cámara con ese ID".to_string());
        }
        self.camaras.remove(&id);
        Ok(())
    }

    pub fn camara(&self, id: IdCamara) -> Option<&Camara> {
        self.camaras.get(&id)
    }

    pub fn camaras(&self) -> Vec<&Camara> {
        self.camaras.values().collect()
    }
    pub fn modificar_ubicacion(&mut self, id: IdCamara, lat: f64, lon: f64) -> Result<(), String> {
        match self.camaras.get_mut(&id) {
            Some(camara) => {
                camara.lat = lat;
                camara.lon = lon;
                Ok(())
            }
            None => Err("No existe una cámara con ese ID".to_string()),
        }
    }

    pub fn modificar_rango(&mut self, id: IdCamara, rango: f64) -> Result<(), String> {
        match self.camaras.get_mut(&id) {
            Some(camara) => {
                camara.rango = rango;
                Ok(())
            }
            None => Err("No existe una cámara con ese ID".to_string()),
        }
    }
}

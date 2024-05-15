use std::collections::HashMap;

use crate::camara::{id::IdCamara, Camara};

/// Estado del sistema de camaras.
/// TODO: Estado de incidentes y de drones
pub struct Estado {
    camaras: HashMap<IdCamara, Camara>,
}

impl Default for Estado {
    fn default() -> Self {
        Self::new()
    }
}

impl Estado {
    pub fn new() -> Self {
        Estado {
            camaras: HashMap::new(),
        }
    }

    /// Al conectar una cámara, se registra.
    /// Todas las cámaras deben tener IDs únicos.
    pub fn conectar_camara(&mut self, camara: Camara) -> Result<(), String> {
        if self.camaras.contains_key(&camara.id) {
            return Err("Ya existe una cámara con ese ID".to_string());
        }
        self.camaras.insert(camara.id, camara);
        Ok(())
    }

    /// Al desconectar una cámara, se borra.
    /// La cámara debe estar conectada para poder desconectarse.
    pub fn desconectar_camara(&mut self, id: IdCamara) -> Result<(), String> {
        if !self.camaras.contains_key(&id) {
            return Err("No existe una cámara con ese ID".to_string());
        }
        self.camaras.remove(&id);
        Ok(())
    }

    /// Encuentra una cámara por su ID.
    pub fn camara(&self, id: IdCamara) -> Option<&Camara> {
        self.camaras.get(&id)
    }

    /// Lista todas las cámaras conectadas.
    pub fn camaras(&self) -> Vec<&Camara> {
        self.camaras.values().collect()
    }

    /// Modifica la ubicación de una cámara ya registrada.
    /// TODO: ESTA BIEN ESTO????????
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

    /// Modifica el rango de una cámara ya registrada.
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

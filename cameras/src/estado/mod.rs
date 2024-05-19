use std::collections::HashMap;

use lib::{coordenadas::Coordenadas, incidente::Incidente};

use crate::camara::{id::IdCamara, Camara};

/// Estado del sistema de camaras.
/// TODO: Estado de incidentes y de drones
pub struct Estado {
    pub camaras: HashMap<IdCamara, Camara>,
    pub incidentes: HashMap<u64, Incidente>,
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
            incidentes: HashMap::new(),
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
    pub fn desconectar_camara(&mut self, id: IdCamara) -> Result<Camara, String> {
        if let Some(camara) = self.camaras.remove(&id) {
            return Ok(camara);
        }

        Err("No existe una cámara con ese ID".to_string())
    }

    /// Encuentra una cámara por su ID.
    pub fn camara(&self, id: IdCamara) -> Option<&Camara> {
        self.camaras.get(&id)
    }

    /// Encuentra una cámara por su ID y devolver una referencia mutable.
    pub fn camara_mut(&mut self, id: IdCamara) -> Option<&mut Camara> {
        self.camaras.get_mut(&id)
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

    pub fn nuevo_incidente(&mut self, incidente: Incidente) {
        for (_id, camara) in self.camaras.iter_mut() {
            let pos_incidente = Coordenadas::from_lat_lon(incidente.lat, incidente.lon);
            if camara.en_rango(&pos_incidente) {
                camara.incidentes.push(incidente.id);
            }
        }

        self.incidentes.insert(incidente.id, incidente);
    }

    pub fn incidente_finalizado(&mut self, id: u64) {
        for camara in self.camaras.values_mut() {
            camara.incidentes.retain(|&i| i != id);
        }
    }

    pub fn incidente(&self, id: u64) -> Option<&Incidente> {
        self.incidentes.get(&id)
    }

    pub fn incidentes_en_rango_de_camara(
        &self,
        id_camara: IdCamara,
        pos: &Coordenadas
    ) -> Vec<u64> {
        self.camaras
            .values()
            .filter(|c| c.id != id_camara && c.en_rango(pos))
            .flat_map(|c| c.incidentes.iter())
            .cloned()
            .collect()
    }
}

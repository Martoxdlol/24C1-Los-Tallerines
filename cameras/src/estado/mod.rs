use std::collections::{HashMap, HashSet};

use lib::{camara::Camara, incidente::Incidente};

/// Estado del sistema de camaras.
/// TODO: Estado de incidentes y de drones
pub struct Estado {
    /// Camaras conectadas al sistema.
    pub camaras: HashMap<u64, Camara>,
    /// Incidentes activos
    pub incidentes: HashMap<u64, Incidente>,
    /// Camaras lindantes a cada camara.
    pub camaras_lindantes: HashMap<u64, HashSet<u64>>,
}

impl Default for Estado {
    fn default() -> Self {
        Self::new()
    }
}

impl Estado {
    pub fn new() -> Self {
        Self {
            camaras: HashMap::new(),
            incidentes: HashMap::new(),
            camaras_lindantes: HashMap::new(),
        }
    }

    pub fn conectar_camara(&mut self, mut camara: Camara) {
        // La cámara ya existe
        if self.camara(camara.id).is_some() {
            return;
        }

        // Establece las camaras lindantes
        self.establecer_camaras_lindantes(&camara);
        // Buscamos los incidentes atendidos por la camara
        camara.incidentes_primarios = self.incidentes_en_rango(&camara);

        let binding = HashSet::new();
        let camaras_lindantes = self
            .camaras_lindantes
            .get(&camara.id)
            .unwrap_or(&binding);
        
        for id in camaras_lindantes.iter() {
            if let Some(camara_lindante) = self.camaras.get_mut(id) {
                // Agrega los incidentes secundarios a la camara, que son los incidentes primarios de las camaras lindantes
                camara
                    .incidentes_secundarios
                    .extend(camara_lindante.incidentes_primarios.clone());
                // Agrega los incidentes secundarios a la camara lindante que son los incidentes primarios de la camara
                camara_lindante
                    .incidentes_secundarios
                    .extend(camara.incidentes_primarios.clone());
            }
        }

        // Agrerga la camara al estado
        self.camaras.insert(camara.id, camara);
    }

    pub fn desconectar_camara(&mut self, id: u64) -> Option<Camara> {
        if let Some(mut camara) = self.camaras.remove(&id) {
            self.restablecer_camaras_lindantes(id);
            camara.incidentes_primarios.clear();
            camara.incidentes_secundarios.clear();
            Some(camara)
        } else {
            None
        }
    }

    pub fn cargar_incidente(&mut self, incidente: Incidente) {
        self.finalizar_incidente(incidente.id);

        for id_camara in self.camaras_en_rango(&incidente) {
            if let Some(camara) = self.camaras.get_mut(&id_camara) {
                camara.incidentes_primarios.insert(incidente.id);
            }

            for id_camara_lindante in self
                .camaras_lindantes
                .get(&id_camara)
                .unwrap_or(&HashSet::new())
            {
                if let Some(camara_lindante) = self.camaras.get_mut(id_camara_lindante) {
                    camara_lindante.incidentes_secundarios.insert(incidente.id);
                }
            }
        }

        self.incidentes.insert(incidente.id, incidente);
    }

    pub fn finalizar_incidente(&mut self, id: u64) {
        if let Some(incidente) = self.incidentes.remove(&id) {
            for id_camara in self.camaras_en_rango(&incidente) {
                if let Some(camara) = self.camaras.get_mut(&id_camara) {
                    camara.incidentes_primarios.remove(&id);
                }

                for id_camara_lindante in self
                    .camaras_lindantes
                    .get(&id_camara)
                    .unwrap_or(&HashSet::new())
                {
                    if let Some(camara_lindante) = self.camaras.get_mut(id_camara_lindante) {
                        camara_lindante.incidentes_secundarios.remove(&id);
                    }
                }
            }
        }
    }

    pub fn modificar_ubicacion_camara(&mut self, id: u64, lat: f64, lon: f64) {
        // Hasta aca borramos la camara, y re acomodamos las lindantes
        if let Some(mut camara) = self.desconectar_camara(id) {
            camara.lat = lat;
            camara.lon = lon;
            self.conectar_camara(camara);
        }
    }

    pub fn modificar_rango_camara(&mut self, id: u64, rango: f64) {
        if let Some(mut camara) = self.desconectar_camara(id) {
            camara.rango = rango;
            self.conectar_camara(camara);
        }
    }

    fn establecer_camaras_lindantes(&mut self, camara: &Camara) {
        // Calcula las camaras lindantes a la camara dada
        let camaras_lindantes: HashSet<u64> = self
            .camaras
            .values()
            .filter(|c| camara.posicion().distancia(&c.posicion()) < camara.rango + c.rango)
            .map(|c| c.id)
            .collect();

        // Almacena la camara dada como lindante para las camaras lindantes
        for id in camaras_lindantes.iter() {
            self.camaras_lindantes
                .entry(*id)
                .or_insert_with(HashSet::new)
                .insert(camara.id);
        }

        // Almacena las camaras lindantes para la camara dada
        self.camaras_lindantes.insert(camara.id, camaras_lindantes);
    }

    fn restablecer_camaras_lindantes(&mut self, id: u64) {
        if let Some(camaras_lindantes) = self.camaras_lindantes.remove(&id) {
            for id_lindante in camaras_lindantes.iter() {
                if let Some(camaras_lindantes_de_camara_lindante) =
                    self.camaras_lindantes.get_mut(id_lindante)
                {
                    camaras_lindantes_de_camara_lindante.remove(&id);
                }
            }
        }
    }

    /// Incidentes que están en el rango de una cámara
    pub fn incidentes_en_rango(&self, camara: &Camara) -> HashSet<u64> {
        let mut incidentes: HashSet<u64> = HashSet::new();
        for incidente in self.incidentes.values() {
            if incidente.posicion().distancia(&camara.posicion()) < camara.rango {
                incidentes.insert(incidente.id);
            }
        }
        incidentes
    }

    pub fn camaras_en_rango(&self, incidente: &Incidente) -> HashSet<u64> {
        let mut camaras: HashSet<u64> = HashSet::new();
        for camara in self.camaras.values() {
            if incidente.posicion().distancia(&camara.posicion()) < camara.rango {
                camaras.insert(camara.id);
            }
        }
        camaras
    }

    /// Devuelve una referencia a la camara con el id dado.
    pub fn camara(&self, id: u64) -> Option<&Camara> {
        self.camaras.get(&id)
    }

    /// Devuelve un vector de camaras
    pub fn camaras(&self) -> Vec<&Camara> {
        self.camaras.values().collect()
    }
}

// /// Agrega una camara al estado.
// pub fn conectar_camara(&mut self, camara: Camara) {
//     if self.camara(camara.id).is_some() {
//         return;
//     }

//     // Establece las camaras lindantes
//     self.establecer_camaras_lindantes(&camara);
//     // Establece el estado de la camara
//     self.camaras.insert(camara.id, camara);
// }

// pub fn desconectar_camara(&mut self, id: u64) {}

// /// Establece el estado de una camara
// pub fn establecer_estado_camara(&mut self, camara: &mut Camara) {
//     let incidentes = self.incidentes_en_rango_de_camara(camara);
//     camara.activa = !incidentes.is_empty();
// }

// /// Calcula y almacena las camaras lindantes a una camara dada.
// ///
// /// Una camara es lindante a otra si la distancia entre sus ubicaciones es menor a la suma de sus rangos.
// /// Es decir, sus rangos estan superpuestos en al menos un punto.
// fn establecer_camaras_lindantes(&self, camara: &Camara) {
//     // Calcula las camaras lindantes a la camara dada
//     let camaras_lindantes = self
//         .camaras
//         .values()
//         .filter(|c| camara.posicion().distancia(&c.posicion()) < camara.rango + c.rango)
//         .map(|c| c.id)
//         .collect();

//     // Almacena las camaras lindantes para la camara dada
//     self.camaras_lindantes.insert(camara.id, camaras_lindantes);

//     // Almacena la camara dada como lindante para las camaras lindantes
//     for id in camaras_lindantes {
//         self.camaras_lindantes
//             .entry(id)
//             .or_insert_with(HashSet::new)
//             .insert(camara.id);
//     }
// }

// /// Camaras que por su posicion atienden al incidente dado
// pub fn camaras_en_rango_de_incidente(&self, incidente: &Incidente) -> HashSet<u64> {
//     // Calcula las camaras que atienden al incidente dado
//     let mut camaras: HashSet<u64> = self
//         .camaras
//         .values()
//         .filter(|c| incidente.posicion().distancia(&c.posicion()) < c.rango)
//         .map(|c| c.id)
//         .collect();

//     // Calcula las camaras lindantes a las camaras que atienden al incidente dado, que tambien lo atienden
//     let mut camaras_lindantes: HashSet<u64> = HashSet::new();
//     for id in camaras {
//         camaras_lindantes.extend(self.camaras_lindantes.get(&id).unwrap_or(&HashSet::new()));
//     }

//     camaras.extend(camaras_lindantes);
//     camaras
// }

// /// Incidentes que por su posicion son atendidos por la camara dada
// pub fn incidentes_en_rango_de_camara(&self, camara: &Camara) -> HashSet<u64> {
//     // La cámara dada y sus camaras lindantes, si cualquiera de estas tiene un incidente en su rango,
//     // la cámara dada debe estar activa
//     let mut camaras = HashSet::new();
//     camaras.insert(camara.id);
//     camaras.extend(
//         self.camaras_lindantes
//             .get(&camara.id)
//             .unwrap_or(&HashSet::new()),
//     );

//     let mut incidentes = HashSet::new();

//     for incidente in self.incidentes.values() {
//         for id_camara in camaras.iter() {
//             if incidente
//                 .posicion()
//                 .distancia(&self.camaras[id_camara].posicion())
//                 < self.camaras[id_camara].rango
//             {
//                 incidentes.insert(incidente.id);
//             }
//         }
//     }

//     incidentes
// }

// /// Devuelve una referencia a la camara con el id dado.
// pub fn camara(&self, id: u64) -> Option<&Camara> {
//     self.camaras.get(&id)
// }

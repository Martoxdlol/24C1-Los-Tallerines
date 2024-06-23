use egui::ahash::HashMap;
use lib::{
    camara::Camara,
    dron::{accion::Accion, Dron},
    incidente::Incidente,
};

#[derive(Clone, Debug)]

/// Estado de la aplicación. Todos los incidentes y cámaras que hay que mostrar.
pub struct Estado {
    drones: HashMap<u64, Dron>,
    camaras: HashMap<u64, Camara>,
    incidentes: HashMap<u64, Incidente>,
    pub conectado: bool,
    pub mensaje_error: Option<String>,
}

impl Default for Estado {
    fn default() -> Self {
        Self::new()
    }
}

impl Estado {
    pub fn new() -> Self {
        Estado {
            drones: HashMap::default(),
            camaras: HashMap::default(),
            incidentes: HashMap::default(),
            conectado: false,
            mensaje_error: None,
        }
    }

    /// Agrega un incidente en el estado.
    ///
    /// Se usa cuando se genera un incidente.
    pub fn cargar_incidente(&mut self, incidente: Incidente) {
        self.incidentes.insert(incidente.id, incidente);
    }

    /// Elimina un incidente en el estado.
    ///
    /// Se usa cuando se finaliza un incidente.
    pub fn finalizar_incidente(&mut self, id: &u64) -> Option<Incidente> {
        self.incidentes.remove(id)
    }

    /// Agrega una cámara en el estado.
    ///
    /// Se usa cuando se conecta una cámara.
    pub fn conectar_camara(&mut self, camara: Camara) {
        self.camaras.insert(camara.id, camara);
    }

    /// Elimina una cámara en el estado.
    ///
    /// Se usa cuando se desconecta una cámara.
    pub fn desconectar_camara(&mut self, id: &u64) -> Option<Camara> {
        self.camaras.remove(id)
    }

    /// Muestra todos los incidentes activos por orden de inicio.
    pub fn incidentes(&self) -> Vec<Incidente> {
        let mut v: Vec<Incidente> = self.incidentes.values().cloned().collect();
        v.sort_by(|a, b| b.inicio.cmp(&a.inicio));
        v
    }

    /// Muestra todas las cámaras.
    pub fn camaras(&self) -> Vec<Camara> {
        self.camaras.values().cloned().collect()
    }

    /// Muestra todos los drones.
    pub fn drones(&self) -> Vec<Dron> {
        self.drones.values().cloned().collect()
    }

    /// Devuelve un dron especifico segun su id.
    pub fn dron(&self, id: u64) -> Option<Dron> {
        self.drones.get(&id).cloned()
    }
    /// Envía un incidente segun su id.
    pub fn incidente(&self, id: u64) -> Option<Incidente> {
        self.incidentes.get(&id).cloned()
    }

    /// Envía una cámara segun su id.
    pub fn camara(&self, id: u64) -> Option<Camara> {
        self.camaras.get(&id).cloned()
    }

    /// Limpia todas las cámaras.
    pub fn limpiar_camaras(&mut self) {
        self.camaras.clear();
    }

    /// Agrega un dron en el estado.
    pub fn cargar_dron(&mut self, dron: Dron) {
        self.drones.insert(dron.id, dron);
    }

    /// Limpia los drones que no enviaron estado en los últimos 10 segundos.
    pub fn limpiar_drones(&mut self) {
        let ahora = chrono::offset::Local::now().timestamp_millis();
        // Eliminar si pasaron más de 10 segundos
        self.drones
            .retain(|_, dron| ahora - dron.envio_ultimo_estado < 10000);
    }

    /// Devuelve los drones que atienden al incidente.
    pub fn drones_incidente(&self, id_incidente: &u64) -> Vec<&Dron> {
        self.drones
            .values()
            .filter(|dron| {
                if let Some(incidente) = &dron.incidente_actual {
                    incidente.id.eq(id_incidente)
                } else {
                    false
                }
            })
            .collect()
    }

    /// Devuelve los drones que no tienen incidentes asignados.
    pub fn drones_disponibles(&self) -> Vec<&Dron> {
        self.drones
            .values()
            .filter(|dron| {
                if let Accion::Espera = dron.accion() {
                    return true;
                }
                false
            })
            .collect()
    }

    /// Devuelve los incidentes que no tienen 2 drones asignados.
    pub fn incidentes_sin_asignar(&self, cantidad_drones: usize) -> Vec<(&Incidente, usize)> {
        self.incidentes
            .values()
            .filter(|incidente| {
                let drones_incidente = self.drones_incidente(&incidente.id);

                drones_incidente.len() < cantidad_drones
            })
            .map(|incidente| {
                let drones_incidente = self.drones_incidente(&incidente.id);
                (incidente, cantidad_drones - drones_incidente.len())
            })
            .collect()
    }

    /// Transforma un incidente a un texto legible por un humano.
    pub fn incidente_a_string(&mut self, id_incidente: &u64) -> String {
        let incidente = self.incidente(*id_incidente);
        if let Some(incidente) = incidente {
            return incidente.detalle.to_string();
        }
        "No se encontró el incidente".to_string()
    }
}

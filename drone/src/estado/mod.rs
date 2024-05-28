use lib::{dron::Dron, incidente::Incidente};
use std::collections::HashMap;

pub struct Estado {
    /// Drones conectados al sistema.
    pub drones: HashMap<u64, Dron>,
    /// Incidentes activos
    pub incidentes: HashMap<u64, Incidente>,
}

impl Default for Estado {
    fn default() -> Self {
        Self::new()
    }
}

impl Estado {
    pub fn new() -> Self {
        Self {
            drones: HashMap::new(),
            incidentes: HashMap::new(),
        }
    }
}

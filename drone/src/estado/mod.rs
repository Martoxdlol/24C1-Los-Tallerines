use std::collections::{HashMap, HashSet};

use lib::{dron::Dron, incidente::Incidente};

/// Estado de la aplicación del dron
pub struct Estado {
    /// Dron de la aplicación
    pub dron: Option<Dron>,
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
            dron: None,
            incidentes: HashMap::new(),
        }
    }
}
use egui::ahash::HashMap;
use lib::{camara::Camara, incidente::Incidente};

#[derive(Clone, Debug)]

/// Estado de la aplicación. Todos los incidentes y cámaras que hay que mostrar.
pub struct Estado {
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

    pub fn incidente_a_string(self: &mut Self, id_incidente: &u64) -> String {
        let incidente = self.incidente(*id_incidente);
        if let Some(incidente) = incidente {
            return format!("{}", incidente.detalle);
        }
        return format!("No se encontró el incidente");
    }
}

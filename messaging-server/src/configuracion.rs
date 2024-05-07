#[derive(Clone)]
pub struct Configuracion {
    pub registros: bool,
}

impl Default for Configuracion {
    fn default() -> Self {
        Self::new()
    }
}

impl Configuracion {
    pub fn new() -> Configuracion {
        Configuracion { registros: true }
    }
}

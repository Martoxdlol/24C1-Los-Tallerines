#[derive(Clone)]
pub struct Configuracion {}

impl Default for Configuracion {
    fn default() -> Self {
        Self::new()
    }
}

impl Configuracion {
    pub fn new() -> Configuracion {
        Configuracion {}
    }
}

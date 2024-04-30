#[derive(Clone)]
pub struct Configuracion {
    pub registros: bool,
}

impl Configuracion {
    pub fn new() -> Configuracion {
        Configuracion { registros: true }
    }
}

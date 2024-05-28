use lib::configuracion::Configuracion;
use std::io;

pub struct AplicacionDron {
    configuracion: Configuracion,
}


impl AplicacionDron {

    pub fn new(configuracion: Configuracion) -> Self {
        AplicacionDron { configuracion }
    } 

    pub fn iniciar(&mut self) -> io::Result<()> {

        Ok(())
    }
}
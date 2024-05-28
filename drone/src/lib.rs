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
        self.cargar_dron()?;


        Ok(())
    }

    fn cargar_dron(&mut self) -> io::Result<()> {
        let ruta_archivo_drones = self
            .configuracion
            .obtener::<String>("drones")
            .unwrap_or("drones.csv".to_string());

        let existe = std::path::Path::new(&ruta_archivo_drones).exists();

        if !existe {
            std::fs::File::create(&ruta_archivo_drones)?;
        }

        Ok(())
    }
}
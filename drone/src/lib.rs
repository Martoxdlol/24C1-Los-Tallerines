use lib::{
    configuracion::Configuracion,
    dron::Dron,
    serializables::guardar::cargar_serializable, serializables::guardar::guardar_serializable,
};
use std::io;

pub struct AplicacionDron {
    configuracion: Configuracion,
    dron: Option<Dron>,
}

impl AplicacionDron {

    pub fn new(configuracion: Configuracion) -> Self {
        AplicacionDron { 
            configuracion,
            dron: None,
        }
    } 

    pub fn iniciar(&mut self) -> io::Result<()> {
        self.cargar_dron()?;


        Ok(())
    }

    fn cargar_dron(&mut self) -> io::Result<()> {
        let ruta_archivo_dron = self
            .configuracion
            .obtener::<String>("drones")
            .unwrap_or("dron.csv".to_string());

        let existe = std::path::Path::new(&ruta_archivo_dron).exists();

        if !existe {
            std::fs::File::create(&ruta_archivo_dron)?;
        }

        let mut dron: Dron = cargar_serializable(&ruta_archivo_dron)?;

        dron.incidentes_cercanos.clear();

        Ok(())
    }
}
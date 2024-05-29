use lib::{configuracion::Configuracion, dron::Dron, serializables::guardar::cargar_serializable};

use std::io;

use crate::estado::Estado;

#[derive(Debug)]
pub struct AplicacionDron {
    configuracion: Configuracion,
    estado: Estado,
}

impl AplicacionDron {
    pub fn new(configuracion: Configuracion, estado: Estado) -> Self {
        AplicacionDron {
            configuracion,
            estado,
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

        let dron: Dron = cargar_serializable(&ruta_archivo_dron)?;

        println!("\nDron: {:?}", dron);

        println!("\nAplicacion: {:?}", self);

        self.estado.incidentes.clear();
        self.estado.iniciar_dron(dron.clone());
        self.estado.descargar_bateria_dron(dron.clone());

        self.estado.dron = Some(dron.clone());

        println!("\nIncidentes en rango: {:?}", dron.incidentes_cercanos);

        Ok(())
    }
}

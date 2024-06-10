use drone::{comunicacion::Comunicacion, sistema::Sistema};
use lib::{configuracion::Configuracion, dron::Dron};

fn main() {
    if let Ok(config) = Configuracion::desde_argv() {
        if let Some(dron) = Dron::crear(&config) {
            let comunicacion = Comunicacion::new(&config);

            let mut sistema = Sistema::new(dron, comunicacion);

            sistema.iniciar();
        } else {
            println!("Faltan parámetros de configuración del dron");
        }
    } else {
        println!("Error al cargar la configuración");
    }
}

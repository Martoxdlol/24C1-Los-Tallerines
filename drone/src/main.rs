use comunicacion::Comunicacion;
use lib::{configuracion::Configuracion, dron::Dron};
use sistema::Sistema;

pub mod comunicacion;
pub mod sistema;

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

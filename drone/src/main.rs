use lib::{configuracion::Configuracion};
use drone::AplicacionDron;

fn main() {
    if let Err(e) = intentar_iniciar_dron() {
        eprintln!("Error al iniciar el dron: {}", e);
        std::process::exit(1);
    }
}

fn intentar_iniciar_dron() -> Result<(), Box<dyn std::error::Error>> {
    let configuracion = Configuracion::desde_argv()?;
    
    let mut aplicacion_dron = AplicacionDron::new(configuracion);

    Ok(aplicacion_dron.iniciar()?)
}

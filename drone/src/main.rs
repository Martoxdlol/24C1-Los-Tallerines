use drone::{estado::Estado};
use lib::configuracion::Configuracion;

fn main() {
    if let Err(e) = intentar_iniciar_dron() {
        eprintln!("Error al iniciar el dron: {}", e);
        std::process::exit(1);
    }
}

fn intentar_iniciar_dron() -> Result<(), Box<dyn std::error::Error>> {
    let estado = Estado::new();

    let configuracion = Configuracion::desde_argv()?;

    Ok(())
}

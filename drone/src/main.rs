use drone::{estado::Estado, interfaz::interfaz, sistema::Sistema};
use lib::configuracion::Configuracion;

fn main() {
    if let Err(e) = intentar_iniciar_sistema() {
        eprintln!("Error al iniciar el sistema: {}", e);
        std::process::exit(1);
    }
}

fn intentar_iniciar_sistema() -> Result<(), Box<dyn std::error::Error>> {
    let estado = Estado::new();
    let (enviar_respuesta, recibir_comandos) = interfaz();

    let configuracion = Configuracion::desde_argv()?;

    Ok(())
}

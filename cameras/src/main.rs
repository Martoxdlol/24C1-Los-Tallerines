// use messaging::client::NATSClient;

use cameras::{estado::Estado, interfaz::interfaz, sistema::Sistema};
use lib::configuracion::ArchivoConfiguracion;

fn main() {
    if let Err(e) = intentar_iniciar_sistema() {
        eprintln!("Error al iniciar el sistema: {}", e);
        std::process::exit(1);
    }
}

fn intentar_iniciar_sistema() -> Result<(), Box<dyn std::error::Error>> {
    let estado = Estado::new();
    let (enviar_respuesta, recibir_comandos) = interfaz();

    let configuracion = ArchivoConfiguracion::desde_argv()?;
    let mut sistema = Sistema::new(estado, configuracion, enviar_respuesta, recibir_comandos);

    Ok(sistema.iniciar()?)
}

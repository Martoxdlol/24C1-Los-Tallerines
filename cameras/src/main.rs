// use messaging::client::NATSClient;

use cameras::{estado::Estado, interfaz::interfaz, sistema::Sistema};

fn main() {
    let estado = Estado::new();
    let (enviar_respuesta, recibir_comandos) = interfaz();

    let mut sistema = Sistema::new(estado, enviar_respuesta, recibir_comandos);

    sistema.iniciar();
}

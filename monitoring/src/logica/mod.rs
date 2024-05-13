use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
};

use self::{comando::Comando, estado::Estado};

pub mod comando;
pub mod estado;

pub fn iniciar_hilo_logica(recibir_comando: Receiver<Comando>, enviar_estado: Sender<Estado>) {
    thread::spawn(move || inicio(recibir_comando, enviar_estado));
}

pub fn inicio(recibir_comando: Receiver<Comando>, enviar_estado: Sender<Estado>) {
    let mut estado = Estado::new();

    loop {
        if let Err(e) = inicio_conexion(&mut estado, &recibir_comando, &enviar_estado) {
            println!("Error: {}", e);
        }
    }
}

pub fn inicio_conexion(
    estado: &mut Estado,
    recibir_comando: &Receiver<Comando>,
    enviar_estado: &Sender<Estado>,
) -> Result<(), String> {
    loop {
        if let Ok(comando) = recibir_comando.try_recv() {
            match comando {
                Comando::NuevoIncidente(incidente) => {
                    estado.agregar_incidente(incidente);
                    let _ = enviar_estado.send(estado.clone());
                }
            }
        }

    }

    Ok(())
}

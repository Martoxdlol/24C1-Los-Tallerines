use monitoring::Aplicacion;
use std::sync::mpsc::channel;

use monitoring::logica::iniciar_hilo_logica;

fn main() -> Result<(), eframe::Error> {
    let (enviar_comando, recibir_comando) = channel();
    let (enviar_estado, recibir_estado) = channel();

    iniciar_hilo_logica(recibir_comando, enviar_estado);

    eframe::run_native(
        "APLICACION DE MONITOREO", // Nombre de la ventana
        Default::default(),
        Box::new(|cc| {
            Box::new(Aplicacion::new(
                enviar_comando,
                recibir_estado,
                cc.egui_ctx.clone(),
            ))
        }),
    )
}

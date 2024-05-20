use monitoring::{logica::intentar_iniciar_sistema, Aplicacion};
use std::{sync::mpsc::channel, thread};

fn main() {
    let (enviar_comando, recibir_comando) = channel();
    let (enviar_estado, recibir_estado) = channel();

    thread::spawn(move || {
        if eframe::run_native(
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
        .is_err()
        {
            std::process::exit(1);
        }
    });

    if let Err(e) = intentar_iniciar_sistema(recibir_comando, enviar_estado) {
        eprintln!("Error al iniciar el sistema: {}", e);
        std::process::exit(1);
    }
}

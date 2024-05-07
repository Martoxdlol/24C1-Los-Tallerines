#[cfg(not(target_arch = "wasm32"))]
use monitoring::Aplicacion;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    // env_logger::init();
    eframe::run_native(
        "APLICACION DE MONITOREO", // Nombre de la ventana
        Default::default(),
        Box::new(|cc| Box::new(Aplicacion::new(cc.egui_ctx.clone()))),
    )
}

#[cfg(not(target_arch = "wasm32"))]
use monitoring::MyApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    // env_logger::init();
    eframe::run_native(
        "MyApp",
        Default::default(),
        Box::new(|cc| Box::new(MyApp::new(cc.egui_ctx.clone()))),
    )
}

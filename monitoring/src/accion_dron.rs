use egui::Ui;
use lib::dron::{accion::Accion, Dron};

use crate::{accion::AccionAplicacion, aplicacion::Aplicacion};

/// Acciones que se pueden realizar con un dron.
pub enum AccionDron {
    Mostrar,          //Por defecto
    VerDetalles(u64), //Mostrar el detalle arriba a la izquierda del dron
}

impl AccionDron {
    /// Ventana para ver los detalles de un dron.
    pub fn ver_detalles_dron(ui: &mut Ui, dron: &Dron, aplicacion: &mut Aplicacion) {
        egui::Window::new("Detalles Dron")
            .collapsible(false)
            .movable(true)
            .resizable(false)
            .collapsible(true)
            .anchor(egui::Align2::LEFT_TOP, [10., 10.])
            .show(ui.ctx(), |ui| {
                ui.label(format!("ID: {}", dron.id));
                ui.label(format!(
                    "Posición: {}, {}",
                    dron.posicion.lat, dron.posicion.lon
                ));
                ui.label(format!("Rango: {} m", dron.rango));
                ui.label(format!("Batería: {:.1}", dron.bateria_actual));
                ui.label(accion_dron_a_string(dron));
                ui.label(format!("Velocidad: {} m/s", dron.velocidad_actual));

                if ui.button("Volver").clicked() {
                    // Cerrar la ventana de detalles.
                    aplicacion.accion = AccionAplicacion::Dron(AccionDron::Mostrar);
                }
            });
    }
}
/// Convierte la acción de un dron a un string.
///
/// Se va a mostrar en la ventana de detalles de un dron.
fn accion_dron_a_string(dron: &Dron) -> String {
    match dron.accion() {
        Accion::Incidente(incidente) => format!("Atendiendo incidente: {}", incidente.detalle),
        Accion::Cargar => "Estado: Cargando batería".to_string(),
        Accion::Espera => "Estado: Esperando".to_string(),
    }
}

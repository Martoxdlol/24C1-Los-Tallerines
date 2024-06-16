use egui::Ui;
use lib::camara::Camara;

use crate::{
    accion::AccionAplicacion,
    aplicacion::Aplicacion,
    listar::estado_camara_a_string,
    logica::{comando::Comando, estado::Estado},
};
/// Acciones que podes hacer con una cámara
pub enum AccionCamara {
    Conectar,
    Modificar(u64),
    CambiarRango(u64),
    CambiarUbicacion(u64),
}

impl AccionCamara {
    /// Ventana para modificar una cámara.
    ///
    /// Accion_camara debe ser Modificar.
    /// Te da todas las opciones para modificar una cámara.
    pub fn modificar_camara(ui: &mut Ui, camara: &Camara, aplicacion: &mut Aplicacion) {
        egui::Window::new("Modificar cámara")
            .collapsible(false)
            .movable(true)
            .resizable(false)
            .collapsible(true)
            .anchor(egui::Align2::LEFT_TOP, [10., 10.])
            .show(ui.ctx(), |ui| {
                ui.label(format!("Cámara: {}", camara.id));
                ui.label(format!("En: {}, {}", camara.lat, camara.lon));
                ui.label(format!("Rango: {}", camara.rango));
                ui.label(format!("Estado: {}", estado_camara_a_string(camara)));
                ui.label(format!(
                    "Atendiendo incidentes: {}",
                    mostrar_incidentes_camara(camara, &mut aplicacion.estado)
                ));

                // Botones para finalizar, modificar detalle, cambiar ubicación y cancelar.
                botones_modificar_camara(ui, camara, aplicacion);
            });
    }

    /// Ventana para modificar una cámara.
    ///
    /// Accion_camara debe ser CambiarUbicacion.
    pub fn modificar_ubicacion_camara(
        ui: &mut Ui,
        camara: &Camara,
        aplicacion: &mut Aplicacion,
        clicked_at: walkers::Position,
    ) {
        egui::Window::new("Modificar ubicación")
            .collapsible(false)
            .movable(true)
            .resizable(false)
            .collapsible(true)
            .anchor(egui::Align2::LEFT_TOP, [10., 10.])
            .show(ui.ctx(), |ui| {
                ui.label(format!(
                    "Mover cámara a: {}, {}",
                    clicked_at.lat(),
                    clicked_at.lon()
                ));
                if ui
                    .add_sized([350., 40.], egui::Button::new("Confirmar"))
                    .clicked()
                {
                    // Mandar comando para cambiar ubicación.
                    Comando::camara_nueva_ubicacion(
                        &aplicacion.enviar_comando,
                        camara.id,
                        clicked_at.lat(),
                        clicked_at.lon(),
                    );

                    // Cambiar la accion para cambiar la ventana de arriba a la izquierda.
                    aplicacion.accion = AccionAplicacion::Camara(AccionCamara::Conectar);
                }
            });
    }

    /// Ventana para modificar el rango de una cámara.
    ///
    /// Accion_camara debe ser CambiarRango.
    pub fn modificar_rango_camara(ui: &mut Ui, camara: &Camara, aplicacion: &mut Aplicacion) {
        egui::Window::new("Modificar rango")
            .collapsible(false)
            .movable(true)
            .resizable(false)
            .collapsible(true)
            .anchor(egui::Align2::LEFT_TOP, [10., 10.])
            .show(ui.ctx(), |ui| {
                ui.add_sized([350., 40.], |ui: &mut Ui| {
                    ui.text_edit_multiline(&mut aplicacion.input_usuario)
                });
                if let Ok(rango) = aplicacion.input_usuario.parse::<f64>() {
                    if ui
                        .add_sized([350., 40.], egui::Button::new("Confirmar"))
                        .clicked()
                    {
                        // Mandar comando para cambiar rango.
                        Comando::camara_nuevo_rango(&aplicacion.enviar_comando, camara.id, rango);

                        // Cambiar la accion para cambiar la ventana de arriba a la izquierda.
                        aplicacion.input_usuario.clear();
                        aplicacion.accion = AccionAplicacion::Camara(AccionCamara::Conectar);
                    }
                }
            });
    }

    /// Ventana para conectar una cámara.
    ///
    /// Accion_camara debe ser Conectar.
    pub fn conectar_camara(
        ui: &mut Ui,
        clicked_at: walkers::Position,
        aplicacion: &mut Aplicacion,
    ) {
        egui::Window::new("Conectar cámara")
            .collapsible(false)
            .movable(true)
            .resizable(false)
            .collapsible(true)
            .anchor(egui::Align2::LEFT_TOP, [10., 10.])
            .show(ui.ctx(), |ui| {
                ui.label(format!(
                    "Conectar cámara en: {}, {}",
                    clicked_at.lat(),
                    clicked_at.lon()
                ));
                ui.add_sized([350., 40.], |ui: &mut Ui| {
                    ui.text_edit_multiline(&mut aplicacion.input_usuario)
                });
                if let Ok(rango) = aplicacion.input_usuario.parse::<f64>() {
                    if ui
                        .add_sized([350., 40.], egui::Button::new("Confirmar"))
                        .clicked()
                    {
                        Comando::conectar_camara(
                            &aplicacion.enviar_comando,
                            clicked_at.lat(),
                            clicked_at.lon(),
                            rango,
                        );

                        aplicacion.input_usuario.clear();
                        aplicacion.accion = AccionAplicacion::Camara(AccionCamara::Conectar);
                    }
                }
            });
    }
}

/// Devuelve los incidentes que esta atendiendo una cámara.
///
/// Se va a mostrar en la ventana de modificar cámara.
fn mostrar_incidentes_camara(camara: &Camara, estado: &mut Estado) -> String {
    let mut incidentes = String::new();
    for incidente in camara.incidentes_primarios.iter() {
        incidentes.push_str(&format!("{}, ", estado.incidente_a_string(incidente)));
        //TODO: Cambiar por detalle.
    }
    for incidente in camara.incidentes_secundarios.iter() {
        incidentes.push_str(&format!("{}, ", estado.incidente_a_string(incidente)));
    }
    incidentes
}

/// Botones para modificar una cámara.
///
/// Son los que aparecen en la ventana de modificar cámara.
fn botones_modificar_camara(ui: &mut Ui, camara: &Camara, aplicacion: &mut Aplicacion) {
    egui::Grid::new("some_unique_id").show(ui, |ui| {
        if ui.button("Desconectar cámara").clicked() {
            Comando::desconectar_camara(&aplicacion.enviar_comando, camara.id);
        }
        if ui.button("Modificar rango").clicked() {
            //aplicacion.detalle_incidente.clone_from(&incidente.detalle);
            aplicacion.accion = AccionAplicacion::Camara(AccionCamara::CambiarRango(camara.id));
        }
        // Para que aparezcan en dos filas
        ui.end_row();

        if ui.button("Modificar ubicacion").clicked() {
            aplicacion.accion = AccionAplicacion::Camara(AccionCamara::CambiarUbicacion(camara.id));
        }
        if ui.button("Cancelar").clicked() {
            //aplicacion.detalle_incidente.clear();
            aplicacion.accion = AccionAplicacion::Camara(AccionCamara::Conectar);
        }
        ui.end_row();
    });
}

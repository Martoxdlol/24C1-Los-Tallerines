use crate::accion::Accion;
use crate::aplicacion::Aplicacion;
use crate::logica::comando::Comando;
use chrono::DateTime;
use egui::Ui;
use lib::incidente::Incidente;

/// Enum para la ventana de la esquina superior izquierda.
pub enum AccionIncidente {
    Crear,
    Modificar(u64),
    CambiarDetalle(u64),
    CambiarUbicacion(u64),
}

impl AccionIncidente {
    /// Ventana para agregar un incidente.
    ///
    /// Accion_incidente debe ser Crear.
    pub fn agregar_incidente(
        ui: &mut Ui,
        clicked_at: walkers::Position,
        aplicacion: &mut Aplicacion,
    ) {
        egui::Window::new("Agregar Incidente")
            .collapsible(false)
            .movable(true)
            .resizable(false)
            .collapsible(true)
            .anchor(egui::Align2::LEFT_TOP, [10., 10.])
            .show(ui.ctx(), |ui| {
                ui.label(format!("En: {}, {}", clicked_at.lat(), clicked_at.lon()));

                ui.add_sized([350., 40.], |ui: &mut Ui| {
                    ui.text_edit_multiline(&mut aplicacion.detalle_incidente)
                });

                if !aplicacion.detalle_incidente.trim().is_empty()
                    && ui
                        .add_sized([350., 40.], egui::Button::new("Confirmar"))
                        .clicked()
                {
                    // Creo el incidente
                    let incidente = Incidente::new(
                        0,
                        aplicacion.detalle_incidente.clone(),
                        clicked_at.lat(),
                        clicked_at.lon(),
                        chrono::offset::Local::now().timestamp_millis() as u64,
                    );

                    aplicacion.detalle_incidente.clear();

                    aplicacion.clicks.clear();

                    // Envio el comando.
                    Comando::nuevo_incidente(&aplicacion.enviar_comando, incidente);
                }
            });
    }

    /// Ventana para modificar un incidente.
    ///
    /// Accion_incidente debe ser Modificar.
    /// Te da todas las opciones para modificar un incidente.
    pub fn modificar_incidente(ui: &mut Ui, incidente: &Incidente, aplicacion: &mut Aplicacion) {
        egui::Window::new("Modificar Incidente")
            .collapsible(false)
            .movable(true)
            .resizable(false)
            .collapsible(true)
            .anchor(egui::Align2::LEFT_TOP, [10., 10.])
            .show(ui.ctx(), |ui| {
                ui.label(format!("Incidente: {}", incidente.detalle));
                ui.label(format!("En: {}, {}", incidente.lat, incidente.lon));
                let dt = DateTime::from_timestamp_millis(incidente.inicio as i64);

                // muestra la fecha del incidente.
                let fecha = match dt {
                    Some(fecha) => fecha.format("%d/%m/%Y %H:%M:%S").to_string(),
                    None => "".to_string(),
                };

                ui.label(fecha);
                ui.label("Estado: activo");

                // Botones para finalizar, modificar detalle, cambiar ubicación y cancelar.
                botones_modificar_inicidente(ui, incidente, aplicacion);
            });
    }

    /// Ventana para cambiar el detalle de un incidente.
    /// Aparece en la esquina superior izquierda si accion_incidente es CambiarDetalle.
    pub fn cambiar_detalle_incidente(
        ui: &mut Ui,
        aplicacion: &mut Aplicacion,
        incidente: &mut Incidente,
    ) {
        egui::Window::new("Modificar Incidente")
            .collapsible(false)
            .movable(true)
            .resizable(false)
            .collapsible(true)
            .anchor(egui::Align2::LEFT_TOP, [10., 10.])
            .show(ui.ctx(), |ui| {
                ui.add_sized([350., 40.], |ui: &mut Ui| {
                    ui.text_edit_multiline(&mut aplicacion.detalle_incidente)
                });

                if !aplicacion.detalle_incidente.trim().is_empty()
                    && ui
                        .add_sized([350., 40.], egui::Button::new("Confirmar"))
                        .clicked()
                {
                    // Creo un incidente nuevo con el detalle cambiado.
                    let mut incidente_nuevo = incidente.clone();
                    incidente_nuevo
                        .detalle
                        .clone_from(&aplicacion.detalle_incidente);
                    aplicacion.detalle_incidente.clear();
                    aplicacion.accion = Accion::Incidente(AccionIncidente::Crear);

                    Comando::modificar_incidente(&aplicacion.enviar_comando, incidente_nuevo);
                }
            });
    }

    /// Ventana para cambiar la ubicación de un incidente.
    ///
    /// Aparece en la esquina superior izquierda si accion_incidente es CambiarUbicacion.
    pub fn cambiar_ubicacion(
        ui: &mut Ui,
        aplicacion: &mut Aplicacion,
        incidente: &mut Incidente,
        clicked_at: walkers::Position,
    ) {
        egui::Window::new("Cambiar ubicación del incidente")
            .collapsible(false)
            .movable(true)
            .resizable(true)
            .collapsible(true)
            .anchor(egui::Align2::LEFT_TOP, [10., 10.])
            .show(ui.ctx(), |ui| {
                ui.label(format!(
                    "Mover incidente a: {}, {}",
                    clicked_at.lat(),
                    clicked_at.lon()
                ));
                if ui
                    .add_sized([350., 40.], egui::Button::new("Confirmar"))
                    .clicked()
                {
                    // Creo un incidente nuevo con la ubicación cambiada.
                    let mut incidente_nuevo = incidente.clone();
                    incidente_nuevo.lat = clicked_at.lat();
                    incidente_nuevo.lon = clicked_at.lon();
                    aplicacion.detalle_incidente.clear();
                    aplicacion.accion = Accion::Incidente(AccionIncidente::Crear);

                    Comando::modificar_incidente(&aplicacion.enviar_comando, incidente_nuevo);
                }
            });
    }
}

fn botones_modificar_inicidente(ui: &mut Ui, incidente: &Incidente, aplicacion: &mut Aplicacion) {
    egui::Grid::new("some_unique_id").show(ui, |ui| {
        if ui.button("Finalizar incidente").clicked() {
            Comando::incidente_finalizado(&aplicacion.enviar_comando, incidente.id);
            aplicacion.detalle_incidente.clear();
            aplicacion.accion = Accion::Incidente(AccionIncidente::Crear);
        }
        if ui.button("Modificar detalle").clicked() {
            aplicacion.detalle_incidente.clone_from(&incidente.detalle);
            aplicacion.accion = Accion::Incidente(AccionIncidente::CambiarDetalle(incidente.id));
        }
        ui.end_row();

        if ui.button("Modificar ubicacion").clicked() {
            aplicacion.accion = Accion::Incidente(AccionIncidente::CambiarUbicacion(incidente.id));
        }
        if ui.button("Cancelar").clicked() {
            aplicacion.detalle_incidente.clear();
            aplicacion.accion = Accion::Incidente(AccionIncidente::Crear);
        }
        ui.end_row();
    });
}

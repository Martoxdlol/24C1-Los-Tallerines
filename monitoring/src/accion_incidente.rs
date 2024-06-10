use crate::accion::AccionAplicacion;
use crate::aplicacion::Aplicacion;
use crate::logica::comando::Comando;
use chrono::DateTime;
use egui::Ui;
use lib::incidente::Incidente;

/// Acciones que podes hacer con un incidente.
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
                    ui.text_edit_multiline(&mut aplicacion.input_usuario)
                });

                if !aplicacion.input_usuario.trim().is_empty()
                    && ui
                        .add_sized([350., 40.], egui::Button::new("Confirmar"))
                        .clicked()
                {
                    // Creo el incidente
                    let incidente = Incidente::new(
                        0,
                        aplicacion.input_usuario.clone(),
                        clicked_at.lat(),
                        clicked_at.lon(),
                        chrono::offset::Local::now().timestamp_millis() as u64,
                    );

                    aplicacion.input_usuario.clear();

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

                let ahora = chrono::offset::Local::now().timestamp_millis() as u64;

                ui.label(fecha);
                ui.label("Estado: activo");

                let drones = aplicacion.estado.drones_incidente(&incidente.id);

                ui.label(format!("Drones atendiendo: {}/2", drones.len()));
                ui.label(format!(
                    "Tiempo atendido: {}/300 segundos",
                    incidente.tiempo_atendido / 1000
                ));

                ui.label(format!(
                    "Tiempo maximo de espera: {}/1200 segundos (20 min)",
                    (ahora - incidente.inicio) / 1000
                ));

                // Botones para modificar el incidente.
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
                    ui.text_edit_multiline(&mut aplicacion.input_usuario)
                });

                if !aplicacion.input_usuario.trim().is_empty()
                    && ui
                        .add_sized([350., 40.], egui::Button::new("Confirmar"))
                        .clicked()
                {
                    // Creo un incidente nuevo con el detalle cambiado.
                    let mut incidente_nuevo = incidente.clone();
                    incidente_nuevo
                        .detalle
                        .clone_from(&aplicacion.input_usuario);
                    aplicacion.input_usuario.clear();
                    aplicacion.accion = AccionAplicacion::Incidente(AccionIncidente::Crear);

                    // Envio el comando
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
                    aplicacion.input_usuario.clear();
                    aplicacion.accion = AccionAplicacion::Incidente(AccionIncidente::Crear);

                    // Piso el incidente anterior con el nuevo (tienen la ubicación diferente)
                    Comando::modificar_incidente(&aplicacion.enviar_comando, incidente_nuevo);
                }
            });
    }
}

/// Botones para modificar un incidente.
///
/// Se muestra en la ventana de modificar incidente.
fn botones_modificar_inicidente(ui: &mut Ui, incidente: &Incidente, aplicacion: &mut Aplicacion) {
    egui::Grid::new("some_unique_id").show(ui, |ui| {
        if ui.button("Finalizar incidente").clicked() {
            Comando::incidente_finalizado(&aplicacion.enviar_comando, incidente.id);
            aplicacion.input_usuario.clear();
            aplicacion.accion = AccionAplicacion::Incidente(AccionIncidente::Crear);
        }
        if ui.button("Modificar detalle").clicked() {
            aplicacion.input_usuario.clone_from(&incidente.detalle);
            aplicacion.accion =
                AccionAplicacion::Incidente(AccionIncidente::CambiarDetalle(incidente.id));
        }
        // Para que aparezcan en dos filas
        ui.end_row();

        if ui.button("Modificar ubicacion").clicked() {
            aplicacion.accion =
                AccionAplicacion::Incidente(AccionIncidente::CambiarUbicacion(incidente.id));
        }
        if ui.button("Cancelar").clicked() {
            aplicacion.input_usuario.clear();
            aplicacion.accion = AccionAplicacion::Incidente(AccionIncidente::Crear);
        }
        ui.end_row();
    });
}

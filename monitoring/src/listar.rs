use crate::accion::Accion;
use crate::accion_dron::AccionDron;
use crate::aplicacion::Aplicacion;
use crate::logica::comando::Comando;
use crate::{accion_camara::AccionCamara, accion_incidente::AccionIncidente};
use egui::{Color32, Ui};
use lib::camara::Camara;
use lib::{camara, incidente::Incidente};
use walkers::Position;

/// Enum para saber si se listan incidentes o cámaras.
pub enum Listar {
    Incidentes,
    Camaras,
    Drones,
}

impl Listar {
    /// Ventana para elegir si listar incidentes o cámaras.
    ///
    /// Aparece en la esquina  inferior derecha.
    pub fn listar(ui: &mut Ui, aplicacion: &mut Aplicacion) {
        egui::Window::new("📝")
            .collapsible(false)
            .movable(true)
            .resizable(true)
            .collapsible(true)
            .anchor(egui::Align2::RIGHT_BOTTOM, [-10., -10.])
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    if ui
                        .add_sized([100., 20.], egui::Button::new("Incidentes"))
                        .clicked()
                    {
                        aplicacion.listar = Listar::Incidentes;
                        aplicacion.accion = Accion::Incidente(AccionIncidente::Crear);
                    }
                    if ui
                        .add_sized([100., 20.], egui::Button::new("Camaras"))
                        .clicked()
                    {
                        aplicacion.listar = Listar::Camaras;
                        aplicacion.accion = Accion::Camara(AccionCamara::Conectar);
                    }
                    if ui
                        .add_sized([100., 20.], egui::Button::new("Drones"))
                        .clicked()
                    {
                        aplicacion.listar = Listar::Drones;
                        aplicacion.accion = Accion::Dron(AccionDron::Mostrar);
                    }
                    if ui
                        .add_sized([100., 20.], egui::Button::new("Salir"))
                        .clicked()
                    {
                        println!("Saliendo");
                        Comando::desconectar(&aplicacion.enviar_comando);
                    }
                });
            });
    }
    /// Lista de cámaras en la esquina superior derecha.
    ///
    /// Muestra el id de la cámara y si está activa o en ahorro.
    /// Listar tiene que estar en Cámaras.
    pub fn listar_camaras(ui: &mut Ui, camaras: &[camara::Camara], aplicacion: &mut Aplicacion) {
        if !camaras.is_empty() {
            egui::Window::new("Lista de cámaras")
                .collapsible(false)
                .movable(true)
                .resizable(true)
                .collapsible(true)
                .anchor(egui::Align2::RIGHT_TOP, [-10., 10.])
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut sorted_camaras = camaras.iter().collect::<Vec<&Camara>>();
                        sorted_camaras.sort_by(|a, b| a.id.cmp(&b.id));

                        for camara in sorted_camaras {
                            let nombre =
                                format!("{}: {}", camara.id, estado_camara_a_string(camara));

                            ui.scope(|ui| {
                                ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                                    Color32::TRANSPARENT;
                                if ui
                                    .add_sized([350., 40.], |ui: &mut Ui| ui.button(nombre))
                                    .clicked()
                                {
                                    // Si clickeas el incidente te lleva a esa posición.
                                    aplicacion
                                        .memoria_mapa
                                        .center_at(Position::from_lat_lon(camara.lat, camara.lon));
                                    // Cambia la AccionIncidente a Modificar.
                                    aplicacion.accion =
                                        Accion::Camara(AccionCamara::Modificar(camara.id));
                                }
                            });
                        }
                    });
                });
        }
    }

    /// Lista de incidentes en la esquina superior derecha.
    ///
    /// Muestra el detalle del incidente.
    /// Listar tiene que estar en Incidentes.
    pub fn listar_incidentes(ui: &mut Ui, incidentes: &[Incidente], aplicacion: &mut Aplicacion) {
        if !incidentes.is_empty() {
            egui::Window::new("Lista de incidentes")
                .collapsible(false)
                .movable(true)
                .resizable(true)
                .collapsible(true)
                .anchor(egui::Align2::RIGHT_TOP, [-10., 10.])
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for incidente in incidentes {
                            let nombre = incidente.detalle.to_string();

                            ui.scope(|ui| {
                                ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                                    Color32::TRANSPARENT;
                                if ui
                                    .add_sized([350., 40.], |ui: &mut Ui| ui.button(nombre))
                                    .clicked()
                                {
                                    // Si clickeas el incidente te lleva a esa posición.
                                    aplicacion.memoria_mapa.center_at(Position::from_lat_lon(
                                        incidente.lat,
                                        incidente.lon,
                                    ));
                                    // Cambia la AccionIncidente a Modificar.
                                    aplicacion.accion =
                                        Accion::Incidente(AccionIncidente::Modificar(incidente.id));
                                }
                            });
                        }
                    });
                });
        }
    }

    pub fn listar_drones(ui: &mut Ui, drones: &[lib::dron::Dron], aplicacion: &mut Aplicacion) {
        if !drones.is_empty() {
            egui::Window::new("Lista de drones")
                .collapsible(false)
                .movable(true)
                .resizable(true)
                .collapsible(true)
                .anchor(egui::Align2::RIGHT_TOP, [-10., 10.])
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for dron in drones {
                            let id = dron.id.to_string();

                            ui.scope(|ui| {
                                ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                                    Color32::TRANSPARENT;
                                if ui
                                    .add_sized([350., 40.], |ui: &mut Ui| ui.button(id))
                                    .clicked()
                                {
                                    // Si clickeas el incidente te lleva a esa posición.
                                    aplicacion.memoria_mapa.center_at(Position::from_lat_lon(
                                        dron.posicion.lat,
                                        dron.posicion.lon,
                                    ));
                                    // Cambia la AccionIncidente a VerDetalle.
                                    //aplicacion.accion =
                                    //    Accion::Dron(AccionDron::VerDetalles(dron.id));
                                }
                            });
                        }
                    });
                });
        }
    }
}

/// Convierte el estado de la cámara a un string.
pub fn estado_camara_a_string(camara: &camara::Camara) -> String {
    if camara.activa() {
        "Activa".to_string()
    } else {
        "Ahorro".to_string()
    }
}

use egui::{Color32, Painter, Response};
use walkers::{
    extras::{Place, Places},
    Plugin, Position, Projector,
};
use crate::incidente::Incidente;

use crate::iconos;

/// Los lugares. Los iconos
pub fn mostrar_incidentes(incidentes: &[Incidente]) -> impl Plugin {
    let mut lugares = Vec::new();


    for incidente in incidentes.iter(){
        lugares.push(Place {
            position: incidente.posicion,
            label: incidente.nombre.clone(),
            symbol: incidente.icono,
            style: incidente.estilo.clone(),
        });
    }
    Places::new(lugares)

}





/// Sample map plugin which draws custom stuff on the map.
pub struct SombreadoCircular {}

impl Plugin for SombreadoCircular {
    fn run(&mut self, response: &Response, painter: Painter, projector: &Projector) {
        // Position of the point we want to put our shapes.
        let position = iconos::incidente();

        // Project it into the position on the screen.
        let position = projector.project(position).to_pos2();

        let radius = 50.;

        let hovered = response
            .hover_pos()
            .map(|hover_pos| hover_pos.distance(position) < radius)
            .unwrap_or(false);

        painter.circle_filled(
            position,
            radius,
            Color32::BLACK.gamma_multiply(if hovered { 0.5 } else { 0.2 }),
        );
    }
}

#[derive(Default, Clone)]
pub struct ClickWatcher {
    pub clicked_at: Option<Position>,
}

impl ClickWatcher { // Donde hiciste click
    pub fn show_position(&mut self, ui: &egui::Ui) {
        if let Some(clicked_at) = self.clicked_at {
            egui::Window::new("Clicked Position")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(egui::Align2::CENTER_BOTTOM, [0., -10.])
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("{:.04} {:.04}", clicked_at.lon(), clicked_at.lat()))
                        .on_hover_text("Posición donde hiciste click");
                    if ui.button("cerrar").clicked() {
                        self.clear()
                    }
                    });
                });
        }
    }

    pub fn clear(&mut self) {
        self.clicked_at = None;
    }
}

impl Plugin for &mut ClickWatcher {
    fn run(&mut self, response: &Response, painter: Painter, projector: &Projector) {
        if !response.changed() && response.clicked_by(egui::PointerButton::Primary) {
            self.clicked_at = response
                .interact_pointer_pos()
                .map(|p| projector.unproject(p - response.rect.center()));
        }

        if let Some(position) = self.clicked_at {
            painter.circle_filled(projector.project(position).to_pos2(), 5.0, Color32::BLUE);
        }
    }
}

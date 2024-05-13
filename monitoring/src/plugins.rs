use crate::{coordenadas::metros_a_pixeles_en_mapa, incidente::Incidente};
use egui::{Color32, Painter, Response, Ui};
use walkers::{
    extras::{Place, Places},
    Plugin, Position, Projector,
};

/// Los lugares. Los iconos
pub fn mostrar_incidentes(incidentes: &[Incidente]) -> impl Plugin {
    let mut lugares = Vec::new();

    for incidente in incidentes.iter() {
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
pub struct SombreadoCircular {
    pub posiciones: Vec<(Position, f32)>,
}

impl Plugin for SombreadoCircular {
    fn run(&mut self, response: &Response, painter: Painter, projector: &Projector) {
        for (posicion, radio_metros) in &self.posiciones {
            // Project it into the position on the screen.
            let posicion_x_y = projector.project(*posicion).to_pos2();

            let radio_como_f64 = *radio_metros as f64;
            let radio = (metros_a_pixeles_en_mapa(radio_como_f64, posicion, projector) as f32)
                * radio_metros;

            println!("XY: {:?}", posicion_x_y);
            println!("Radio: {}", radio);

            let flotar = response
                .hover_pos()
                .map(|hover_pos| hover_pos.distance(posicion_x_y) < radio)
                .unwrap_or(false);

            painter.circle_filled(
                posicion_x_y,
                radio,
                Color32::BLACK.gamma_multiply(if flotar { 0.5 } else { 0.2 }),
            );
        }
    }
}

#[derive(Default, Clone)]
pub struct ClickWatcher {
    pub clicked_at: Option<Position>,
}

fn posicion_click(ui: &mut Ui, clicked_at: Position) {
    ui.label(format!("{:.04} {:.04}", clicked_at.lon(), clicked_at.lat()))
        .on_hover_text("Posición donde hiciste click");
}

fn click_cerrar(ui: &mut Ui, clickwatcher: &mut ClickWatcher) {
    if ui.button("Cerrar").clicked() {
        clickwatcher.clear()
    }
}

impl ClickWatcher {
    // Donde hiciste click
    pub fn mostrar_posicion(&mut self, ui: &Ui) {
        if let Some(clicked_at) = self.clicked_at {
            egui::Window::new("Posicion clickeada")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(egui::Align2::CENTER_BOTTOM, [0., -10.])
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        posicion_click(ui, clicked_at);
                        click_cerrar(ui, self);
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

        if let Some(posicion) = self.clicked_at {
            painter.circle_filled(projector.project(posicion).to_pos2(), 5.0, Color32::BLUE);
        }
    }
}

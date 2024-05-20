use crate::coordenadas::metros_a_pixeles_en_mapa;
use egui::{Color32, Painter, Response, Ui};
use lib::{camara::Camara, coordenadas::Coordenadas, incidente::Incidente};
use walkers::{
    extras::{Place, Places, Style},
    Plugin, Position, Projector,
};

/// Los lugares. Los iconos
pub fn mostrar_incidentes(incidentes: &[Incidente]) -> impl Plugin {
    let mut lugares = Vec::new();

    for incidente in incidentes.iter() {
        lugares.push(Place {
            position: Position::from_lat_lon(incidente.posicion().lat, incidente.posicion().lon),
            label: incidente.detalle.clone(),
            symbol: '🚨',
            style: Style::default(),
        });
    }
    Places::new(lugares)
}

/// Los lugares. Los iconos
pub fn mostrar_camaras(camaras: &[Camara]) -> impl Plugin {
    let mut lugares = Vec::new();

    for camara in camaras.iter() {
        let mut estado = "Ahorro";
        let mut symbol = '📷';
        if camara.activa() {
            estado = "Activa";
            symbol = '📸';
        }

        lugares.push(Place {
            position: Position::from_lat_lon(camara.posicion().lat, camara.posicion().lon),
            label: format!("Id: {}, Estado: {}", camara.id, estado),
            symbol,
            style: Style::default(),
        });
    }
    Places::new(lugares)
}

/// Sample map plugin which draws custom stuff on the map.
pub struct SombreadoCircular {
    pub posiciones: Vec<(Coordenadas, f64, bool)>,
}

impl Plugin for SombreadoCircular {
    fn run(&mut self, response: &Response, painter: Painter, projector: &Projector) {
        for (coordenadas, radio_metros, activa) in &self.posiciones {

            let posicion = Position::from_lat_lon(coordenadas.lat, coordenadas.lon);
            // Project it into the position on the screen.
            let posicion_x_y = projector.project(posicion).to_pos2();

            //let radio_como_f64 = *radio_metros as f64;
            let radio = (metros_a_pixeles_en_mapa(&posicion, projector)  * radio_metros) as f32;

            let mouse_encima = response
                .hover_pos()
                .map(|hover_pos| hover_pos.distance(posicion_x_y) < radio)
                .unwrap_or(false);

            painter.circle_filled(
                posicion_x_y,
                radio,
                color_circulo(activa.clone(), mouse_encima),
            );
        }
    }
}

fn color_circulo(activa: bool, mouse_encima: bool) -> Color32 {
    if activa {
        Color32::LIGHT_GREEN.gamma_multiply(if mouse_encima { 0.5 } else { 0.2 })
    } else {
        Color32::BLACK.gamma_multiply(if mouse_encima { 0.5 } else { 0.2 })
    }
}

#[derive(Default, Clone)]
pub struct ClickWatcher {
    pub clicked_at: Option<Position>,
}

fn posicion_click(ui: &mut Ui, clicked_at: Position) {
    ui.label(format!("lat, lon: {:.04} {:.04}", clicked_at.lat(), clicked_at.lon()))
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

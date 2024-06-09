use crate::coordenadas::{self, metros_a_pixeles_en_mapa};
use egui::{Color32, FontId, Painter, Response, Stroke, Ui};
use lib::{camara::Camara, coordenadas::Coordenadas, dron::Dron, incidente::Incidente};
use walkers::{
    extras::{Place, Places, Style},
    Plugin, Position, Projector,
};

/// Muestra los incidentes en el mapa
pub fn mostrar_incidentes(incidentes: &[Incidente]) -> impl Plugin {
    let mut lugares = Vec::new();

    for incidente in incidentes.iter() {
        lugares.push(Place {
            position: Position::from_lat_lon(incidente.posicion().lat, incidente.posicion().lon),
            label: incidente.detalle.clone(),
            symbol: '🚨',
            style: estilo_incidente(),
        });
    }
    Places::new(lugares)
}

/// El estilo especial de los incidentes. Que sean rojos, letra más grande, etc.
fn estilo_incidente() -> Style {
    Style {
        label_font: FontId::proportional(15.),
        label_color: Color32::WHITE,
        symbol_background: Color32::RED,
        ..Default::default()
    }
}

/// Muestra las camaras en el mapa según su estado
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

fn estilo_dron() -> Style {
    Style {
        symbol_font: FontId::proportional(20.),
        symbol_stroke: Stroke::new(16., Color32::from_rgb(255, 51, 236)),
        symbol_color: Color32::WHITE,
        ..Default::default()
    }
}

pub fn mostrar_drones(drones: &[Dron]) -> impl Plugin {
    let mut lugares = Vec::new();

    for dron in drones.iter() {
        let symbol = '🚁';

        let diferencial_tiempo = chrono::Local::now().timestamp_millis() - dron.envio_ultimo_estado;

        let coordenadas = dron.predecir_posicion((diferencial_tiempo as f64) / 1000.);

        lugares.push(Place {
            position: Position::from_lat_lon(coordenadas.lat, coordenadas.lon),
            label: format!("Id: {}", dron.id),
            symbol,
            style: estilo_dron(),
        });
    }
    Places::new(lugares)
}

/// Sombreado circular en el mapa. Sirve para marcar el rango de las cámaras.
///
/// Futuramente va a marcar el rango de los drones.
pub struct SombreadoCircular {
    pub posiciones: Vec<(Coordenadas, f64, bool)>,
}

/// Muestra el sombreado circular en el mapa según el estado de las cámaras.
impl Plugin for SombreadoCircular {
    fn run(&mut self, response: &Response, painter: Painter, projector: &Projector) {
        for (coordenadas, radio_metros, activa) in &self.posiciones {
            let posicion = Position::from_lat_lon(coordenadas.lat, coordenadas.lon);
            // Project it into the position on the screen.
            let posicion_x_y = projector.project(posicion).to_pos2();

            //let radio_como_f64 = *radio_metros as f64;
            let radio = (metros_a_pixeles_en_mapa(&posicion, projector) * radio_metros) as f32;

            let mouse_encima = response
                .hover_pos()
                .map(|hover_pos| hover_pos.distance(posicion_x_y) < radio)
                .unwrap_or(false);

            painter.circle_filled(posicion_x_y, radio, color_circulo(*activa, mouse_encima));
        }
    }
}

/// Color del círculo según si la cámara está activa o no.
fn color_circulo(activa: bool, mouse_encima: bool) -> Color32 {
    if activa {
        Color32::LIGHT_GREEN.gamma_multiply(if mouse_encima { 0.4 } else { 0.3 })
    } else {
        Color32::BLACK.gamma_multiply(if mouse_encima { 0.4 } else { 0.3 })
    }
}

#[derive(Default, Clone)]
/// Posición donde hiciste click dentro de la aplicación.
pub struct ClickWatcher {
    pub clicked_at: Option<Position>,
}

/// Muestra la posición donde hiciste click en la aplicación.
fn posicion_click(ui: &mut Ui, clicked_at: Position) {
    ui.label(format!(
        "lat, lon: {:.04} {:.04}",
        clicked_at.lat(),
        clicked_at.lon()
    ))
    .on_hover_text("Posición donde hiciste click");
}

/// Botón para cerrar la ventana posición_click.
fn click_cerrar(ui: &mut Ui, clickwatcher: &mut ClickWatcher) {
    if ui.button("Cerrar").clicked() {
        clickwatcher.clear()
    }
}

impl ClickWatcher {
    // Cartel donde aoarece la posición clikeada y un botón para cerrarlo.
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

    /// Limpia la posición clickeada.
    pub fn clear(&mut self) {
        self.clicked_at = None;
    }
}

impl Plugin for &mut ClickWatcher {
    /// Muestra un puntero con la posición donde hiciste click.
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

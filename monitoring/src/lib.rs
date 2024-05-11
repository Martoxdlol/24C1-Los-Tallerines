mod botones;
mod iconos;
mod incidente;
mod dron;
mod plugins;
mod proveer_carto;

use std::collections::HashMap;

//use iconos::incidente;

use egui::{Context, Ui};
use incidente::Incidente;
use dron::Dron;
use proveer_carto::MapaCarto;
use walkers::{HttpOptions, Map, MapMemory, Tiles, TilesManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
    OpenStreetMap,
    Geoportal,
    CartoMaps,
}

fn http_options() -> HttpOptions {
    HttpOptions {
        // Not sure where to put cache on Android, so it will be disabled for now.
        cache: if cfg!(target_os = "android") || std::env::var("NO_HTTP_CACHE").is_ok() {
            None
        } else {
            Some(".cache".into())
        },
        ..Default::default()
    }
}

fn estilo_mapa(contexto: Context) -> HashMap<Provider, Box<dyn TilesManager + Send>> {
    let mut providers: HashMap<Provider, Box<dyn TilesManager + Send>> = HashMap::default();

    providers.insert(
        Provider::CartoMaps,
        Box::new(Tiles::with_options(
            MapaCarto {},
            http_options(),
            contexto.to_owned(),
        )),
    );

    providers
}

pub struct Aplicacion {
    opciones_mapa: HashMap<Provider, Box<dyn TilesManager + Send>>,
    estilo_mapa_elegido: Provider,
    memoria_mapa: MapMemory, // guarda el zoom, la posicion, el centro del mapa
    nombre_incidente: String,
    nombre_dron: String,
    clicks: plugins::ClickWatcher,
    incidentes: Vec<incidente::Incidente>,
    drones: Vec<dron::Dron>,
}

impl Aplicacion {
    pub fn new(contexto: Context) -> Self {
        egui_extras::install_image_loaders(&contexto);

        Self {
            opciones_mapa: estilo_mapa(contexto.to_owned()),
            estilo_mapa_elegido: Provider::CartoMaps,
            memoria_mapa: MapMemory::default(),
            clicks: Default::default(),
            nombre_incidente: String::new(),
            nombre_dron: String::new(),
            incidentes: Vec::new(),
            drones: Vec::new(),
        }
    }
}

impl eframe::App for Aplicacion {
    fn update(&mut self, contexto: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = egui::Frame {
            fill: contexto.style().visuals.panel_fill,
            ..Default::default()
        };

        egui::CentralPanel::default()
            .frame(frame)
            .show(contexto, |ui| {
                // coordenadas iniciales
                let posicion_inicial = iconos::obelisco();

                let mapa = self
                    .opciones_mapa
                    .get_mut(&self.estilo_mapa_elegido)
                    .unwrap()
                    .as_mut();

                let mapa_a_mostrar = Map::new(Some(mapa), &mut self.memoria_mapa, posicion_inicial);

                // HARDCODEO INCIDENTE
                let mapa_final = mapa_a_mostrar
                    .with_plugin(plugins::mostrar_incidentes(&self.incidentes))
                    .with_plugin(plugins::SombreadoCircular {
                        posiciones: self.incidentes.iter().map(|i| (i.posicion, 50.)).collect(),
                    })
                    .with_plugin(&mut self.clicks);

                /*
                // HARDCODEO DRON
                let mapa_final = mapa_a_mostrar
                    .with_plugin(plugins::mostrar_drones(&self.drones))
                    .with_plugin(plugins::SombreadoCircular {
                        posiciones: self.drones.iter().map(|i| (i.posicion, 50.)).collect(),
                    })
                    .with_plugin(&mut self.clicks);
                */

                // Draw the map widget.
                ui.add(mapa_final);

                // Draw utility windows.
                {
                    use botones::*;

                    zoom(ui, &mut self.memoria_mapa);
                    ir_a_posicion_inicial(ui, &mut self.memoria_mapa);
                    self.clicks.mostrar_posicion(ui);
                }

                if let Some(clicked_at) = self.clicks.clicked_at {
                    egui::Window::new("Agregar Incidente")
                        .collapsible(false)
                        .movable(true)
                        .resizable(false)
                        .collapsible(true)
                        .anchor(egui::Align2::LEFT_TOP, [10., 10.])
                        .show(ui.ctx(), |ui| {
                            ui.label(format!("En: {}, {}", clicked_at.lat(), clicked_at.lon()));

                            ui.add_sized([350., 40.], |ui: &mut Ui| {
                                ui.text_edit_multiline(&mut self.nombre_incidente)
                            });

                            if !self.nombre_incidente.trim().is_empty()
                                && ui
                                    .add_sized([350., 40.], egui::Button::new("Confirmar"))
                                    .clicked()
                            {
                                let incidente = Incidente::new(
                                    clicked_at.lon(),
                                    clicked_at.lat(),
                                    self.nombre_incidente.clone(),
                                );

                                self.nombre_incidente.clear();

                                self.clicks.clear();

                                self.incidentes.push(incidente);
                            }
                        });
                }

                /*
                    egui::Window::new("Agregar Dron")
                        .collapsible(false)
                        .movable(true)
                        .resizable(false)
                        .collapsible(true)
                        .anchor(egui::Align2::LEFT_TOP, [10., 10.])
                        .show(ui.ctx(), |ui| {
                            ui.label(format!("En: {}, {}", clicked_at.lat(), clicked_at.lon()));

                            ui.add_sized([350., 40.], |ui: &mut Ui| {
                                ui.text_edit_multiline(&mut self.nombre_dron)
                            });

                            if !self.nombre_dron.trim().is_empty()
                                && ui
                                    .add_sized([350., 40.], egui::Button::new("Confirmar"))
                                    .clicked()
                            {
                                let dron = Dron::new(
                                    clicked_at.lon(),
                                    clicked_at.lat(),
                                    self.nombre_dron.clone(),
                                );

                                self.nombre_dron.clear();

                                self.clicks.clear();

                                self.drones.push(dron);
                            }
                        });
                    */
            });
    }
}

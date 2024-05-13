mod botones;
mod iconos;
mod incidente;
mod camara;
mod dron;
mod plugins;
mod proveer_carto;
mod coordenadas;

use std::collections::HashMap;

//use iconos::incidente;

use egui::{Context, Ui};
use incidente::Incidente;


use proveer_carto::MapaCarto;
use walkers::{HttpOptions, Map, MapMemory, Tiles, TilesManager};
use crate::plugins::ClickWatcher;

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
    nombre_incidente: String, // El input de cuando lo creas.
    clicks: plugins::ClickWatcher,
    incidentes: Vec<incidente::Incidente>,
    drones: Vec<dron::Dron>,
    camaras: Vec<camara::Camara>,
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
            incidentes: Vec::new(),
            drones: Vec::new(),
            camaras: Vec::new(),
        }
    }
}

fn agregar_incidente(ui: &mut Ui, clicked_at: walkers::Position, aplicacion: &mut Aplicacion) {
    egui::Window::new("Agregar Incidente")
        .collapsible(false)
        .movable(true)
        .resizable(false)
        .collapsible(true)
        .anchor(egui::Align2::LEFT_TOP, [10., 10.])
        .show(ui.ctx(), |ui| {
            ui.label(format!("En: {}, {}", clicked_at.lat(), clicked_at.lon()));

            ui.add_sized([350., 40.], |ui: &mut Ui| {
                ui.text_edit_multiline(&mut aplicacion.nombre_incidente)
            });

            if !aplicacion.nombre_incidente.trim().is_empty()
                && ui
                    .add_sized([350., 40.], egui::Button::new("Confirmar"))
                    .clicked()
            {
                let incidente = Incidente::new(
                    clicked_at.lon(),
                    clicked_at.lat(),
                    aplicacion.nombre_incidente.clone(),
                );

                aplicacion.nombre_incidente.clear();

                aplicacion.clicks.clear();

                aplicacion.incidentes.push(incidente);
            }
        });
}

fn mostrado_de_incidentes<'a>(mapa_a_mostrar: Map<'a, 'a, 'a>, incidentes: &[Incidente], clicks: &'a mut ClickWatcher) -> Map<'a, 'a, 'a> {
    mapa_a_mostrar
        .with_plugin(plugins::mostrar_incidentes(incidentes))
        .with_plugin(plugins::SombreadoCircular {
            posiciones: incidentes.iter().map(|i| (i.posicion, 50.)).collect(),
        })
        .with_plugin(clicks)
}

fn lista_de_incidentes_actuales(ui: &mut Ui, incidentes: &[Incidente]) {
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
                        let nombre = format!("{}: {}", incidente.icono, incidente.nombre);

                        ui.add_sized([350., 40.], |ui: &mut Ui| {
                            ui.label(nombre)
                        });
                    }
                });
            });
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
                let mapa_final = mostrado_de_incidentes(mapa_a_mostrar, &self.incidentes, &mut self.clicks);            

                /*
                // HARDCODEO DRON
                let mapa_final = mapa_a_mostrar
                    .with_plugin(plugins::mostrar_drones(&self.drones))
                    .with_plugin(plugins::SombreadoCircular {
                        posiciones: self.drones.iter().map(|i| (i.posicion, 50.)).collect(),
                    })
                    .with_plugin(&mut self.clicks);
                */

                /*
                // HARDCODEO CAMARA
                let mapa_final = mapa_a_mostrar
                    .with_plugin(plugins::mostrar_camaras(&self.camaras))
                    .with_plugin(plugins::SombreadoCircular {
                        posiciones: self.camaras.iter().map(|i| (i.posicion, 50.)).collect(),
                    })
                    .with_plugin(&mut self.clicks);
                */

                // Draw the map widget.
                ui.add(mapa_final);

                // Draw utility windows.
                {
                    use botones::*;

                    zoom(ui, &mut self.memoria_mapa);
                    self.clicks.mostrar_posicion(ui);
                }

                if let Some(clicked_at) = self.clicks.clicked_at {
                    agregar_incidente(ui, clicked_at, self);
                }

                lista_de_incidentes_actuales(ui, &self.incidentes);

                /*
                if !self.drones.is_empty() {
                    egui::Window::new("Lista de drones")
                        .collapsible(false)
                        .movable(true)
                        .resizable(false)
                        .collapsible(true)
                        .anchor(egui::Align2::RIGHT_TOP, [10., 110.])
                        .show(ui.ctx(), |ui| {
                        });
                }

                if !self.camaras.is_empty() {
                    egui::Window::new("Lista de camaras")
                        .collapsible(false)
                        .movable(true)
                        .resizable(false)
                        .collapsible(true)
                        .anchor(egui::Align2::RIGHT_TOP, [10., 210.])
                        .show(ui.ctx(), |ui| {
                        });
                }
                */

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

                /*
                    egui::Window::new("Agregar Camara")
                        .collapsible(false)
                        .movable(true)
                        .resizable(false)
                        .collapsible(true)
                        .anchor(egui::Align2::LEFT_TOP, [10., 10.])
                        .show(ui.ctx(), |ui| {
                            ui.label(format!("En: {}, {}", clicked_at.lat(), clicked_at.lon()));

                            ui.add_sized([350., 40.], |ui: &mut Ui| {
                                ui.text_edit_multiline(&mut self.nombre_camara)
                            });

                            if !self.nombre_camara.trim().is_empty()
                                && ui
                                    .add_sized([350., 40.], egui::Button::new("Confirmar"))
                                    .clicked()
                            {
                                let camara = Camara::new(
                                    clicked_at.lon(),
                                    clicked_at.lat(),
                                    self.nombre_camara.clone(),
                                );

                                self.nombre_camara.clear();

                                self.clicks.clear();

                                self.camaras.push(camara);
                            }
                        });
                */

            });
    }
}

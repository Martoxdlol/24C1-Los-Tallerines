mod botones;
mod camara;
mod coordenadas;
mod dron;
mod iconos;
mod incidente;
pub mod logica;
mod plugins;
mod proveer_carto;

use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

//use iconos::incidente;

use egui::{Context, Ui};
use incidente::Incidente;

use crate::plugins::ClickWatcher;
use logica::{comando::Comando, estado::Estado};
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
    nombre_incidente: String, // El input de cuando lo creas.
    clicks: plugins::ClickWatcher,
    estado: Estado,
    recibir_estado: Receiver<Estado>,
    enviar_comando: Sender<Comando>,
}

impl Aplicacion {
    pub fn new(
        enviar_comando: Sender<Comando>,
        recibir_estado: Receiver<Estado>,
        contexto: Context,
    ) -> Self {
        egui_extras::install_image_loaders(&contexto);

        Self {
            opciones_mapa: estilo_mapa(contexto.to_owned()),
            estilo_mapa_elegido: Provider::CartoMaps,
            memoria_mapa: MapMemory::default(),
            clicks: Default::default(),
            nombre_incidente: String::new(),
            estado: Estado::new(),
            recibir_estado,
            enviar_comando,
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

                Comando::nuevo_incidente(&aplicacion.enviar_comando, incidente);
            }
        });
}

fn mostrado_de_incidentes<'a>(
    mapa_a_mostrar: Map<'a, 'a, 'a>,
    incidentes: &[Incidente],
    clicks: &'a mut ClickWatcher,
) -> Map<'a, 'a, 'a> {
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

                        ui.add_sized([350., 40.], |ui: &mut Ui| ui.label(nombre));
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

        if let Ok(estado) = self.recibir_estado.try_recv() {
            self.estado = estado;
        }

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

                let mapa_final = mostrado_de_incidentes(
                    mapa_a_mostrar,
                    self.estado.incidentes(),
                    &mut self.clicks,
                );

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

                lista_de_incidentes_actuales(ui, self.estado.incidentes());

                egui::Context::request_repaint(contexto)
            });
    }
}

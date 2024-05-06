mod iconos;
mod plugins;
mod botones;
mod proveer_carto;
mod incidente;

use std::collections::HashMap;

//use iconos::incidente;

use proveer_carto::MapaCarto;
use egui::Context;
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
    clicks: plugins::ClickWatcher,
}

impl Aplicacion {
    pub fn new(contexto: Context) -> Self {
        egui_extras::install_image_loaders(&contexto);


        Self {
            opciones_mapa: estilo_mapa(contexto.to_owned()),
            estilo_mapa_elegido: Provider::CartoMaps,
            memoria_mapa: MapMemory::default(),
            clicks: Default::default(),
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
                let incidente = incidente::Incidente::new(-58.3816, -34.6062);
                let vector_incidentes = vec![incidente];


                let mapa_final = mapa_a_mostrar
                    .with_plugin(plugins::mostrar_incidentes(vector_incidentes))
                    .with_plugin(plugins::SombreadoCircular {})
                    .with_plugin(&mut self.clicks);

                // Draw the map widget.
                ui.add(mapa_final);

                // Draw utility windows.
                {
                    use botones::*;

                    zoom(ui, &mut self.memoria_mapa);
                    ir_a_posicion_inicial(ui, &mut self.memoria_mapa);
                    self.clicks.show_position(ui);
                }
            });
    }
}

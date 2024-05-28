mod accion_incidente;
mod botones_mover_mapa;
mod coordenadas;
mod iconos;
mod listar;
pub mod logica;
mod plugins;
mod proveer_carto;
use crate::plugins::ClickWatcher;
use accion_incidente::AccionIncidente;
use egui::Context;
use listar::Listar;
use logica::{comando::Comando, estado::Estado};
use proveer_carto::MapaCarto;
use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};
use walkers::{HttpOptions, Map, MapMemory, Tiles, TilesManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
    CartoMaps,
}

/// Opciones de HTTP para el proveedor de mapas.
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

/// Estilos de mapa disponibles.
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

/// Aplicación de monitoreo. UI.
pub struct Aplicacion {
    opciones_mapa: HashMap<Provider, Box<dyn TilesManager + Send>>,
    estilo_mapa_elegido: Provider,
    memoria_mapa: MapMemory, // guarda el zoom, la posicion, el centro del mapa
    detalle_incidente: String, // El input de cuando lo creas.
    clicks: plugins::ClickWatcher,
    estado: Estado,
    recibir_estado: Receiver<Estado>,
    enviar_comando: Sender<Comando>,
    listar: Listar,
    accion_incidente: AccionIncidente,
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
            detalle_incidente: String::new(),
            estado: Estado::new(),
            recibir_estado,
            enviar_comando,
            listar: Listar::Incidentes,
            accion_incidente: AccionIncidente::Crear,
        }
    }
}

/// Muestra los incidentes y las cámaras en el mapa.
fn mostrado_incidentes_y_camaras<'a>(
    mapa_a_mostrar: Map<'a, 'a, 'a>,
    estado: &Estado,
    clicks: &'a mut ClickWatcher,
) -> Map<'a, 'a, 'a> {
    mapa_a_mostrar
        .with_plugin(plugins::mostrar_incidentes(&estado.incidentes()))
        .with_plugin(plugins::mostrar_camaras(&estado.camaras()))
        .with_plugin(plugins::SombreadoCircular {
            posiciones: estado
                .camaras()
                .iter()
                .map(|i| (i.posicion(), i.rango, i.activa()))
                .collect(),
        })
        .with_plugin(clicks)
}

impl eframe::App for Aplicacion {
    /// Lo que ocurre cada vez que actualizamos
    fn update(&mut self, contexto: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = egui::Frame {
            fill: contexto.style().visuals.panel_fill,
            ..Default::default()
        };

        // Intentar recibir estado actualizado del sistema
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

                let mapa_final =
                    mostrado_incidentes_y_camaras(mapa_a_mostrar, &self.estado, &mut self.clicks);

                ui.add(mapa_final);

                {
                    use botones_mover_mapa::*;

                    zoom(ui, &mut self.memoria_mapa);
                    self.clicks.mostrar_posicion(ui);
                }

                // Que mostrar en la esquina superior izquierda.
                match self.accion_incidente {
                    AccionIncidente::Crear => {
                        if let Some(clicked_at) = self.clicks.clicked_at {
                            AccionIncidente::agregar_incidente(ui, clicked_at, self);
                        }
                    }
                    AccionIncidente::Modificar(id) => {
                        if let Some(incidente) = self.estado.incidente(id) {
                            AccionIncidente::modificar_incidente(ui, &incidente, self);
                        }
                    }
                    AccionIncidente::CambiarDetalle(id) => {
                        if let Some(mut incidente) = self.estado.incidente(id) {
                            AccionIncidente::cambiar_detalle_incidente(ui, self, &mut incidente);
                        }
                    }
                    AccionIncidente::CambiarUbicacion(id) => {
                        if let Some(mut incidente) = self.estado.incidente(id) {
                            if let Some(clicked_at) = self.clicks.clicked_at {
                                AccionIncidente::cambiar_ubicacion(
                                    ui,
                                    self,
                                    &mut incidente,
                                    clicked_at,
                                );
                            }
                        }
                    }
                }
                // Esquina inferior derecha.
                Listar::listar(ui, self);

                // Que mostrar en la esquina superior derecha.
                match self.listar {
                    Listar::Incidentes => {
                        Listar::listar_incidentes(ui, &self.estado.incidentes(), self)
                    }
                    Listar::Camaras => Listar::listar_camaras(ui, &self.estado.camaras()),
                }

                egui::Context::request_repaint(contexto)
            });
    }
}

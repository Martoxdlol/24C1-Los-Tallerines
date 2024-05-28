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
use chrono::DateTime;
use egui::{Context, Ui};
use lib::incidente::Incidente;
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

/// Ventana para agregar un incidente.
///
/// Accion_incidente debe ser Crear.
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
                ui.text_edit_multiline(&mut aplicacion.detalle_incidente)
            });

            if !aplicacion.detalle_incidente.trim().is_empty()
                && ui
                    .add_sized([350., 40.], egui::Button::new("Confirmar"))
                    .clicked()
            {
                // Creo el incidente
                let incidente = Incidente::new(
                    0,
                    aplicacion.detalle_incidente.clone(),
                    clicked_at.lat(),
                    clicked_at.lon(),
                    chrono::offset::Local::now().timestamp_millis() as u64,
                );

                aplicacion.detalle_incidente.clear();

                aplicacion.clicks.clear();

                // Envio el comando.
                Comando::nuevo_incidente(&aplicacion.enviar_comando, incidente);
            }
        });
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

/// Ventana para modificar un incidente.
///
/// Accion_incidente debe ser Modificar.
/// Te da todas las opciones para modificar un incidente.
fn modificar_incidente(ui: &mut Ui, incidente: &Incidente, aplicacion: &mut Aplicacion) {
    egui::Window::new("Modificar Incidente")
        .collapsible(false)
        .movable(true)
        .resizable(false)
        .collapsible(true)
        .anchor(egui::Align2::LEFT_TOP, [10., 10.])
        .show(ui.ctx(), |ui| {
            ui.label(format!("Incidente: {}", incidente.detalle));
            ui.label(format!("En: {}, {}", incidente.lat, incidente.lon));
            let dt = DateTime::from_timestamp_millis(incidente.inicio as i64);

            // muestra la fecha del incidente.
            let fecha = match dt {
                Some(fecha) => fecha.format("%d/%m/%Y %H:%M:%S").to_string(),
                None => "".to_string(),
            };

            ui.label(fecha);
            ui.label("Estado: activo");

            // Botones para finalizar, modificar detalle, cambiar ubicación y cancelar.
            egui::Grid::new("some_unique_id").show(ui, |ui| {
                if ui.button("Finalizar incidente").clicked() {
                    Comando::incidente_finalizado(&aplicacion.enviar_comando, incidente.id);
                    aplicacion.detalle_incidente.clear();
                    aplicacion.accion_incidente = AccionIncidente::Crear;
                }
                if ui.button("Modificar detalle").clicked() {
                    aplicacion.detalle_incidente.clone_from(&incidente.detalle);
                    aplicacion.accion_incidente = AccionIncidente::CambiarDetalle(incidente.id);
                }
                ui.end_row();

                if ui.button("Modificar ubicacion").clicked() {
                    aplicacion.accion_incidente = AccionIncidente::CambiarUbicacion(incidente.id);
                }
                if ui.button("Cancelar").clicked() {
                    aplicacion.detalle_incidente.clear();
                    aplicacion.accion_incidente = AccionIncidente::Crear;
                }
                ui.end_row();
            });
        });
}

/// Ventana para cambiar el detalle de un incidente.
/// Aparece en la esquina superior izquierda si accion_incidente es CambiarDetalle.
fn cambiar_detalle_incidente(ui: &mut Ui, aplicacion: &mut Aplicacion, incidente: &mut Incidente) {
    egui::Window::new("Modificar Incidente")
        .collapsible(false)
        .movable(true)
        .resizable(false)
        .collapsible(true)
        .anchor(egui::Align2::LEFT_TOP, [10., 10.])
        .show(ui.ctx(), |ui| {
            ui.add_sized([350., 40.], |ui: &mut Ui| {
                ui.text_edit_multiline(&mut aplicacion.detalle_incidente)
            });

            if !aplicacion.detalle_incidente.trim().is_empty()
                && ui
                    .add_sized([350., 40.], egui::Button::new("Confirmar"))
                    .clicked()
            {
                // Creo un incidente nuevo con el detalle cambiado.
                let mut incidente_nuevo = incidente.clone();
                incidente_nuevo
                    .detalle
                    .clone_from(&aplicacion.detalle_incidente);
                aplicacion.detalle_incidente.clear();
                aplicacion.accion_incidente = AccionIncidente::Crear;

                Comando::incidente_finalizado(&aplicacion.enviar_comando, incidente.id);
                Comando::nuevo_incidente(&aplicacion.enviar_comando, incidente_nuevo);
            }
        });
}

/// Ventana para cambiar la ubicación de un incidente.
///
/// Aparece en la esquina superior izquierda si accion_incidente es CambiarUbicacion.
fn cambiar_ubicacion(
    ui: &mut Ui,
    aplicacion: &mut Aplicacion,
    incidente: &mut Incidente,
    clicked_at: walkers::Position,
) {
    egui::Window::new("Cambiar ubicación del incidente")
        .collapsible(false)
        .movable(true)
        .resizable(true)
        .collapsible(true)
        .anchor(egui::Align2::LEFT_TOP, [10., 10.])
        .show(ui.ctx(), |ui| {
            ui.label(format!(
                "Mover incidente a: {}, {}",
                clicked_at.lat(),
                clicked_at.lon()
            ));
            if ui
                .add_sized([350., 40.], egui::Button::new("Confirmar"))
                .clicked()
            {
                // Creo un incidente nuevo con la ubicación cambiada.
                let mut incidente_nuevo = incidente.clone();
                incidente_nuevo.lat = clicked_at.lat();
                incidente_nuevo.lon = clicked_at.lon();
                aplicacion.detalle_incidente.clear();
                aplicacion.accion_incidente = AccionIncidente::Crear;

                Comando::incidente_finalizado(&aplicacion.enviar_comando, incidente.id);
                Comando::nuevo_incidente(&aplicacion.enviar_comando, incidente_nuevo);
            }
        });
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
                            agregar_incidente(ui, clicked_at, self);
                        }
                    }
                    AccionIncidente::Modificar(id) => {
                        if let Some(incidente) = self.estado.incidente(id) {
                            modificar_incidente(ui, &incidente, self);
                        }
                    }
                    AccionIncidente::CambiarDetalle(id) => {
                        if let Some(mut incidente) = self.estado.incidente(id) {
                            cambiar_detalle_incidente(ui, self, &mut incidente);
                        }
                    }
                    AccionIncidente::CambiarUbicacion(id) => {
                        if let Some(mut incidente) = self.estado.incidente(id) {
                            if let Some(clicked_at) = self.clicks.clicked_at {
                                cambiar_ubicacion(ui, self, &mut incidente, clicked_at);
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

use crate::accion_incidente::AccionIncidente;
use crate::botones_mover_mapa;
use crate::estilo_mapa;
use crate::iconos;
use crate::listar::Listar;
use crate::logica::comando::Comando;
use crate::logica::estado::Estado;
use crate::mostrado_incidentes_y_camaras;
use crate::plugins;
use crate::Provider;
use egui::Context;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use walkers::{Map, MapMemory, TilesManager};

/// Aplicación de monitoreo. UI.
pub struct Aplicacion {
    pub opciones_mapa: HashMap<Provider, Box<dyn TilesManager + Send>>,
    pub estilo_mapa_elegido: Provider,
    pub memoria_mapa: MapMemory, // guarda el zoom, la posicion, el centro del mapa
    pub detalle_incidente: String, // El input de cuando lo creas.
    pub clicks: plugins::ClickWatcher,
    pub estado: Estado,
    pub recibir_estado: Receiver<Estado>,
    pub enviar_comando: Sender<Comando>,
    pub listar: Listar,
    pub accion_incidente: AccionIncidente,
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

    fn actualizar_aplicacion(&mut self, ui: &mut egui::Ui) {
        self.actualizar_mapa(ui);

        {
            use botones_mover_mapa::*;

            zoom(ui, &mut self.memoria_mapa);
            self.clicks.mostrar_posicion(ui);
        }

        self.mostrar_esquina_superior_derecha(ui);

        // Esquina inferior derecha.
        Listar::listar(ui, self);

        self.mostrar_esquina_inferior_derecha(ui);
    }

    fn actualizar_mapa(&mut self, ui: &mut egui::Ui) {
        // coordenadas iniciales
        let posicion_inicial = iconos::obelisco();

        let mapa = self
            .opciones_mapa
            .get_mut(&self.estilo_mapa_elegido)
            .unwrap()
            .as_mut();

        let mapa_a_mostrar = Map::new(Some(mapa), &mut self.memoria_mapa, posicion_inicial);

        let mapa_final = mostrado_incidentes_y_camaras(mapa_a_mostrar, &self.estado, &mut self.clicks);

        ui.add(mapa_final);
    }

    /// Que mostrar en la esquina superior izquierda.
    fn mostrar_esquina_superior_derecha(&mut self, ui: &mut egui::Ui) {
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
    }

    /// Que mostrar en la esquina superior derecha.
    fn mostrar_esquina_inferior_derecha(&mut self, ui: &mut egui::Ui) {
        match self.listar {
            Listar::Incidentes => {
                Listar::listar_incidentes(ui, &self.estado.incidentes(), self)
            }
            Listar::Camaras => Listar::listar_camaras(ui, &self.estado.camaras()),
        }
    }
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
                self.actualizar_aplicacion(ui);

                egui::Context::request_repaint(contexto)
            });
    }
}

use crate::accion::AccionAplicacion;
use crate::accion_camara::AccionCamara;
use crate::accion_dron::AccionDron;
use crate::accion_incidente::AccionIncidente;
use crate::botones_mover_mapa;
use crate::iconos;
use crate::listar::Listar;
use crate::logica::comando::Comando;
use crate::logica::estado::Estado;
use crate::plugins;
use crate::provider::estilo_mapa;
use crate::provider::Provider;

use egui::Context;
use egui::Ui;
use lib::configuracion::Configuracion;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use walkers::{Map, MapMemory, TilesManager};

/// Muestra los incidentes y las cámaras en el mapa.
fn mostrado_incidentes_y_camaras<'a>(
    mapa_a_mostrar: Map<'a, 'a, 'a>,
    estado: &Estado,
    clicks: &'a mut plugins::ClickWatcher,
) -> Map<'a, 'a, 'a> {
    mapa_a_mostrar
        .with_plugin(plugins::mostrar_centros_carga(&estado.drones()))
        .with_plugin(plugins::mostrar_camaras(&estado.camaras()))
        .with_plugin(plugins::mostrar_drones(&estado.drones()))
        .with_plugin(plugins::mostrar_incidentes(&estado.incidentes()))
        .with_plugin(plugins::SombreadoCircular {
            posiciones: estado
                .camaras()
                .iter()
                .map(|i| (i.posicion(), i.rango, i.activa()))
                .collect(),
        })
        .with_plugin(clicks)
}

/// Aplicación de monitoreo. UI.
pub struct Aplicacion {
    pub opciones_mapa: HashMap<Provider, Box<dyn TilesManager + Send>>,
    pub estilo_mapa_elegido: Provider,
    pub memoria_mapa: MapMemory, // guarda el zoom, la posicion, el centro del mapa
    pub input_usuario: String,   // El input de cuando lo creas.
    pub clicks: plugins::ClickWatcher,
    pub estado: Estado,
    pub recibir_estado: Receiver<Estado>,
    pub enviar_comando: Sender<Comando>,
    pub listar: Listar,
    pub accion: AccionAplicacion,
    pub configuracion: Configuracion,
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
            input_usuario: String::new(),
            estado: Estado::new(),
            recibir_estado,
            enviar_comando,
            listar: Listar::Incidentes,
            accion: AccionAplicacion::Incidente(AccionIncidente::Crear),
            configuracion: Configuracion::desde_argv().unwrap_or_default(),
        }
    }

    /// Se llama en cada frame y se encarga de dibujar en pantalla la aplicación.
    fn mostrar_aplicacion(&mut self, ui: &mut egui::Ui) {
        self.mostrar_mapa(ui);

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

    /// Mostrar el mapa en pantalla
    fn mostrar_mapa(&mut self, ui: &mut egui::Ui) {
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
    }

    /// Que mostrar en la esquina superior izquierda.
    fn mostrar_esquina_superior_derecha(&mut self, ui: &mut egui::Ui) {
        match self.accion {
            AccionAplicacion::Incidente(AccionIncidente::Crear) => {
                if let Some(clicked_at) = self.clicks.clicked_at {
                    AccionIncidente::agregar_incidente(ui, clicked_at, self);
                }
            }
            AccionAplicacion::Incidente(AccionIncidente::Modificar(id)) => {
                if let Some(incidente) = self.estado.incidente(id) {
                    AccionIncidente::modificar_incidente(ui, &incidente, self);
                }
            }
            AccionAplicacion::Incidente(AccionIncidente::CambiarDetalle(id)) => {
                if let Some(mut incidente) = self.estado.incidente(id) {
                    AccionIncidente::cambiar_detalle_incidente(ui, self, &mut incidente);
                }
            }
            AccionAplicacion::Incidente(AccionIncidente::CambiarUbicacion(id)) => {
                if let Some(mut incidente) = self.estado.incidente(id) {
                    if let Some(clicked_at) = self.clicks.clicked_at {
                        AccionIncidente::cambiar_ubicacion(ui, self, &mut incidente, clicked_at);
                    }
                }
            }
            AccionAplicacion::Camara(AccionCamara::Modificar(id)) => {
                if let Some(camara) = self.estado.camara(id) {
                    AccionCamara::modificar_camara(ui, &camara, self);
                }
            }
            AccionAplicacion::Camara(AccionCamara::CambiarUbicacion(id)) => {
                if let Some(camara) = self.estado.camara(id) {
                    if let Some(clicked_at) = self.clicks.clicked_at {
                        AccionCamara::modificar_ubicacion_camara(ui, &camara, self, clicked_at);
                    }
                }
            }
            AccionAplicacion::Camara(AccionCamara::CambiarRango(id)) => {
                if let Some(camara) = self.estado.camara(id) {
                    AccionCamara::modificar_rango_camara(ui, &camara, self);
                }
            }
            AccionAplicacion::Camara(AccionCamara::Conectar) => {
                if let Some(clicked_at) = self.clicks.clicked_at {
                    AccionCamara::conectar_camara(ui, clicked_at, self);
                }
            }
            AccionAplicacion::Dron(AccionDron::Mostrar) => {
                if let Some(clicked_at) = self.clicks.clicked_at {
                    AccionIncidente::agregar_incidente(ui, clicked_at, self);
                }
            }
            AccionAplicacion::Dron(AccionDron::VerDetalles(id)) => {
                if let Some(dron) = self.estado.dron(id) {
                    AccionDron::ver_detalles_dron(ui, &dron, self);
                }
            }
        }
    }

    /// Que mostrar en la esquina inferior derecha.
    fn mostrar_esquina_inferior_derecha(&mut self, ui: &mut egui::Ui) {
        match self.listar {
            Listar::Incidentes => Listar::listar_incidentes(ui, &self.estado.incidentes(), self),
            Listar::Camaras => Listar::listar_camaras(ui, &self.estado.camaras(), self),
            Listar::Drones => Listar::listar_drones(ui, &self.estado.drones(), self),
        }
    }

    /// Pantalla de autenticación
    ///
    /// Aparece siempre cuando inicias el programa
    fn mostrar_autenticacion(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Iniciar sesión")
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
            .show(ui.ctx(), |ui| {
                let mut user = self
                    .configuracion
                    .obtener::<String>("user")
                    .unwrap_or("".to_string());
                let mut pass = self
                    .configuracion
                    .obtener::<String>("pass")
                    .unwrap_or("".to_string());
                let mut direccion = self
                    .configuracion
                    .obtener::<String>("direccion")
                    .unwrap_or("127.0.0.1".to_string());
                let mut puerto = self
                    .configuracion
                    .obtener::<String>("puerto")
                    .unwrap_or("4222".to_string());
                let mut archivo = self
                    .configuracion
                    .obtener::<String>("incidentes")
                    .unwrap_or("incidentes.csv".to_string());

                ui.label("Usuario");
                ui.add_sized([350., 20.], |ui: &mut Ui| {
                    ui.text_edit_singleline(&mut user)
                });

                ui.label("Contraseña");
                ui.add_sized([350., 20.], |ui: &mut Ui| {
                    ui.text_edit_singleline(&mut pass)
                });

                ui.label("Dirección");
                ui.add_sized([350., 20.], |ui: &mut Ui| {
                    ui.text_edit_singleline(&mut direccion)
                });

                ui.label("Puerto");
                ui.add_sized([350., 20.], |ui: &mut Ui| {
                    ui.text_edit_singleline(&mut puerto)
                });

                ui.label("Archvio de incidentes");
                ui.add_sized([350., 20.], |ui: &mut Ui| {
                    ui.text_edit_singleline(&mut archivo)
                });

                self.configuracion.setear("user", user);
                self.configuracion.setear("pass", pass);
                self.configuracion.setear("direccion", direccion);
                self.configuracion.setear("puerto", puerto);
                self.configuracion.setear("incidentes", archivo);

                if let Some(error) = self.estado.mensaje_error.as_ref() {
                    if !error.is_empty() {
                        ui.label(
                            egui::RichText::new(error)
                                .heading()
                                .color(egui::Color32::from_rgb(255, 40, 40)),
                        );
                    }
                }

                if ui
                    .add_sized([350., 40.], egui::Button::new("Conectar al sistema"))
                    .clicked()
                {
                    Comando::configurar(&self.enviar_comando, self.configuracion.clone());
                }
            });
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
        while let Ok(estado) = self.recibir_estado.try_recv() {
            self.estado = estado;
        }

        egui::CentralPanel::default()
            .frame(frame)
            .show(contexto, |ui| {
                if !self.estado.conectado {
                    self.mostrar_autenticacion(ui);
                } else {
                    self.mostrar_aplicacion(ui);
                }

                egui::Context::request_repaint(contexto)
            });
    }
}

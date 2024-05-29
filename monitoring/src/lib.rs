pub mod accion_incidente;
pub mod aplicacion;
pub mod botones_mover_mapa;
pub mod coordenadas;
pub mod iconos;
pub mod listar;
pub mod logica;
pub mod plugins;
pub mod proveer_carto;
use crate::plugins::ClickWatcher;
use accion_incidente::AccionIncidente;
use aplicacion::Aplicacion;
use egui::Context;
use listar::Listar;
use logica::estado::Estado;
use proveer_carto::MapaCarto;
use std::collections::HashMap;
use walkers::{HttpOptions, Map, Tiles, TilesManager};

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
pub fn estilo_mapa(contexto: Context) -> HashMap<Provider, Box<dyn TilesManager + Send>> {
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

/// Muestra los incidentes y las cámaras en el mapa.
pub fn mostrado_incidentes_y_camaras<'a>(
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

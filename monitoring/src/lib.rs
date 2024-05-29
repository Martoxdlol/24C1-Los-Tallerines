pub mod accion_incidente;
pub mod aplicacion;
pub mod botones_mover_mapa;
pub mod coordenadas;
pub mod iconos;
pub mod listar;
pub mod logica;
pub mod plugins;
pub mod proveer_carto;
pub mod provider;

use crate::plugins::ClickWatcher;
use aplicacion::Aplicacion;
use logica::estado::Estado;
use walkers::Map;

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

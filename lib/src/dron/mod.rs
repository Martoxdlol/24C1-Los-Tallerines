pub mod accion;

use accion::Accion;

use crate::{
    configuracion::Configuracion, coordenadas::Coordenadas, incidente::Incidente,
    serializables::Serializable,
};

pub struct Dron {
    pub id: u64,
    pub rango: f64,
    pub central_de_carga: Coordenadas,
    pub punto_de_espera: Coordenadas,
    pub velocidad_maxima: f64,
    pub velocidad_descarga_bateria: f64,
    pub posicion: Coordenadas,
    pub direccion_actual: f64,
    pub bateria_actual: f64,
    pub velocidad_actual: f64,
    pub incidente_actual: Option<Incidente>,
}

impl Dron {
    pub fn crear(config: &Configuracion) -> Option<Self> {
        let central_de_carga_lat = config.obtener("central_de_carga.lat")?;
        let central_de_carga_lon = config.obtener("central_de_carga.lon")?;

        let central_de_carga =
            Coordenadas::from_lat_lon(central_de_carga_lat, central_de_carga_lon);

        let punto_de_espera_lat = config.obtener("punto_de_espera.lat")?;
        let punto_de_espera_lon = config.obtener("punto_de_espera.lon")?;

        let punto_de_espera = Coordenadas::from_lat_lon(punto_de_espera_lat, punto_de_espera_lon);

        Some(Dron {
            id: config.obtener("id")?,
            rango: config.obtener("rango").unwrap_or(1500.),
            bateria_actual: config.obtener("bateria").unwrap_or(100.),
            central_de_carga,
            direccion_actual: config.obtener("direccion").unwrap_or(0.),
            incidente_actual: None,
            posicion: Coordenadas::from_lat_lon(
                config.obtener("lat").unwrap_or(punto_de_espera_lat),
                config.obtener("lon").unwrap_or(punto_de_espera_lon),
            ),
            punto_de_espera,
            velocidad_maxima: config.obtener("rapidez").unwrap_or(2.5),
            velocidad_actual: config.obtener("rapidez").unwrap_or(0.),
            velocidad_descarga_bateria: config
                .obtener("velocidad_descarga_bateria")
                .unwrap_or(1. / 3600.),
        })
    }
}

impl Dron {
    pub fn accion(&self) -> Accion {
        if self.bateria_actual < 10. {
            return Accion::Cargar;
        }

        if let Some(incidente) = &self.incidente_actual {
            return Accion::Incidente(incidente.clone());
        }

        return Accion::Espera;
    }
}

impl Serializable for Dron {
    fn deserializar(data: &[u8]) -> Result<Self, crate::serializables::error::DeserializationError>
    where
        Self: Sized,
    {
        todo!()
    }

    fn serializar(&self) -> Vec<u8> {
        todo!()
    }
}

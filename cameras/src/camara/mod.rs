pub mod id;

use std::fmt::{self, Display, Formatter};

use lib::{
    coordenadas::Coordenadas, incidente::Incidente,
    serializables::camara_serializable::CamaraSerializable,
};

use self::id::IdCamara;

/// El modelo de una cámara de seguridad.
#[derive(Clone)]
pub struct Camara {
    pub id: IdCamara,
    pub lat: f64,
    pub lon: f64,
    pub rango: f64,
    pub incidentes: Vec<u64>, // Incidentes en los que esta trabajando la cámara. Cuando el incidente finaliza, se borra.
}

/// Los posibles estados de una cámara de seguridad.
pub enum EstadoCamara {
    AhorroEnergia,
    Activo,
}

impl Camara {
    /// Genera una nueva camara de seguridad.
    pub fn new(id: u64, lat: f64, lon: f64, rango: f64) -> Self {
        Camara {
            id,
            lat,
            lon,
            rango,
            incidentes: Vec::new(),
        }
    }

    /// Devuelve el estado de la cámara.
    /// Si la camara no esta trabajando en un incidente, esta en Ahorro de Energia.
    pub fn estado(&self) -> EstadoCamara {
        if self.incidentes.is_empty() {
            return EstadoCamara::AhorroEnergia;
        }
        EstadoCamara::Activo
    }

    pub fn serializable(&self) -> CamaraSerializable {
        CamaraSerializable {
            id: self.id,
            lat: self.lat,
            lon: self.lon,
            rango: self.rango,
        }
    }

    pub fn en_rango(&self, coordenadas: &Coordenadas) -> bool {
        let distancia = coordenadas.distancia(&Coordenadas::from_lat_lon(self.lat, self.lon));
        distancia <= self.rango
    }
}

impl Display for EstadoCamara {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            EstadoCamara::AhorroEnergia => write!(f, "Ahorro de energía"),
            EstadoCamara::Activo => write!(f, "Activo"),
        }
    }
}

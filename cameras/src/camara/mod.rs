pub mod id;

use std::fmt::{self, Display, Formatter};

use lib::incidente::Incidente;

use self::id::IdCamara;

#[derive(Clone)]
pub struct Camara {
    pub id: IdCamara,
    pub lat: f64,
    pub lon: f64,
    pub rango: f64,
    incidentes: Vec<Incidente>,
}

pub enum EstadoCamara {
    AhorroEnergia,
    Activo,
}

impl Camara {
    pub fn new(id: u64, lat: f64, lon: f64, rango: f64) -> Self {
        Camara {
            id,
            lat,
            lon,
            rango,
            incidentes: Vec::new(),
        }
    }

    pub fn estado(&self) -> EstadoCamara {
        if self.incidentes.is_empty() {
            return EstadoCamara::AhorroEnergia;
        }
        EstadoCamara::Activo
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

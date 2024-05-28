use std::collections::HashSet;
use std::time::Duration;

pub enum EstadoDron {
    EnEspera,
    VolviendoACentroDeOperacion,
    YendoAIncidente,
    AtendiendoIncidente,
    YendoACentral,
    CargandoEnCentral,
}

pub struct Dron {
    pub id: u64,
    pub latitud: f64,
    pub longitud: f64,
    pub rango: f64,
    estado: EstadoDron,
    direccion: f64, // En grados
    velocidad: f64,
    duracion_bateria: Duration,
    incidentes_cercanos: HashSet<u64>,
    central: (f64, f64), // Latitud y longitud
    centro_area_operacion: (f64, f64) // Latitud y longitud
}


impl Dron {
    pub fn new(id: u64, latitud: f64, longitud: f64, rango: f64, estado: EstadoDron, direccion: f64, velocidad: f64, duracion_bateria: Duration, central: (f64, f64), centro_area_operacion: (f64, f64)) -> Self {
        Dron {
            id,
            latitud,
            longitud,
            rango,
            estado,
            direccion,
            velocidad,
            duracion_bateria, 
            incidentes_cercanos: HashSet::new(),
            central, 
            centro_area_operacion,
        }
    }
}

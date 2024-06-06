use crate::coordenadas::Coordenadas;

pub struct Dron {
    pub lat: f64,
    pub lon: f64,
    pub direccion: f64,
    pub bateria: f64,
    pub rapidez: f64,
    pub destino: Coordenadas,
}

impl Dron {
    pub fn new(lat: f64, lon: f64, bateria: f64, direccion: f64) -> Self {
        Dron {
            lat,
            lon,
            direccion,
            bateria,
            rapidez: 2.5,
            destino: Coordenadas::from_lat_lon(lat, lon),
        }
    }
}

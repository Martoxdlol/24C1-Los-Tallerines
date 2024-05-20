use walkers::{extras::Style, Position};

#[derive(Clone)]
pub struct MarcadorIncidente {
    pub posicion: Position,
    pub nombre: String,
    pub icono: char,
    pub estilo: Style,
}
impl MarcadorIncidente {
    pub fn new(longitud: f64, latitud: f64, nombre: String) -> Self {
        MarcadorIncidente {
            posicion: Position::from_lon_lat(longitud, latitud),
            nombre,
            icono: '🚨',
            estilo: Style::default(),
        }
    }
}

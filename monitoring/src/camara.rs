use walkers::{extras::Style, Position};

pub struct Camara {
    pub posicion: Position,
    pub nombre: String,
    pub icono: char,
    pub estilo: Style,
    pub radio: f64,
}
impl Camara {
    pub fn new(longitud: f64, latitud: f64, nombre: String) -> Self {
        Camara {
            posicion: Position::from_lon_lat(longitud, latitud),
            nombre,
            icono: '📹',
            estilo: Style::default(),
            radio: 150.,
        }
    }
}

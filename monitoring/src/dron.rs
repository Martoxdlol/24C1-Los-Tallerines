use walkers::{extras::Style, Position};

#[derive(Clone)]
pub struct Dron {
    pub posicion: Position,
    pub nombre: String,
    pub icono: char,
    pub estilo: Style,
    pub radio: f64,
}
impl Dron {
    pub fn new(longitud: f64, latitud: f64, nombre: String) -> Self {
        Dron {
            posicion: Position::from_lon_lat(longitud, latitud),
            nombre,
            icono: '🚁',
            estilo: Style::default(),
            radio: 150.,
        }
    }
}

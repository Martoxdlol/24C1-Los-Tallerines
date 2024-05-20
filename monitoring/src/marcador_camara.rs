use walkers::{extras::Style, Position};

#[derive(Clone)]
pub struct MarcadorCamara {
    pub posicion: Position,
    pub id: u64,
    pub icono: char,
    pub estilo: Style,
    pub radio: f64,
}
impl MarcadorCamara {
    pub fn new(longitud: f64, latitud: f64, id: u64) -> Self {
        MarcadorCamara {
            posicion: Position::from_lon_lat(longitud, latitud),
            id,
            icono: '📹',
            estilo: Style::default(),
            radio: 150.,
        }
    }
}

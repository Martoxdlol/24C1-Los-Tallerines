use walkers::{extras::Style, Position};


pub struct Incidente {
    pub posicion: Position,
    pub nombre : String,
    pub icono: char,
    pub estilo: Style,
    pub radio: f64,
}
impl Incidente {
    pub fn new(longitud: f64, latitude: f64) -> Self {
        Incidente{
            posicion: Position::from_lon_lat(longitud, latitude),
                nombre: "Incidente".to_owned(),
                icono: '🚨',
                estilo: Style::default(),
                radio: 150.
        }
    }
}

    
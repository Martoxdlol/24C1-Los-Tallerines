use walkers::{Position, Projector};

pub fn grados_a_pixeles(position: &Position, projector: &Projector) -> f32 {
    let p2 = Position::from_lat_lon(position.lat(), position.lon() + 1.);

    let p1_en_pantalla = projector.project(position.to_owned()).to_pos2();
    let p2_en_pantalla = projector.project(p2).to_pos2();

    p2_en_pantalla.x - p1_en_pantalla.x
}

pub fn distancia_coordenadas(p1: &Position, p2: &Position) -> f64 {
    let lat1 = p1.lat();
    let lon1 = p1.lon();

    let lat2 = p2.lat();
    let lon2 = p2.lon();

    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();

    let a = (d_lat / 2.0).sin() * (d_lat / 2.0).sin()
        + lat1.to_radians().cos()
            * lat2.to_radians().cos()
            * (d_lon / 2.0).sin()
            * (d_lon / 2.0).sin();

    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    6_371_000.0 * c
}

pub fn metros_a_pixeles_en_mapa(metros: f64, position: &Position, projector: &Projector) -> f64 {
    let p2 = Position::from_lat_lon(position.lat(), position.lon() + 1.);
    let metros_un_grado = distancia_coordenadas(position, &p2);

    let grados_en_un_metro = 1.0 / metros_un_grado;

    let pixeles_por_grado = grados_a_pixeles(position, projector);

    let pixeles_por_metro = (pixeles_por_grado as f64) * grados_en_un_metro;
    pixeles_por_metro
}

#[cfg(test)]
mod tests {
    use walkers::Position;

    use crate::coordenadas::distancia_coordenadas;

    #[test]
    fn distancia_entre_obelisco_y_luna_park() {
        let obelisco = Position::from_lon_lat(-58.3816, -34.6037);
        let luna_park = Position::from_lon_lat(-58.3689, -34.6020);

        let distancia = distancia_coordenadas(&obelisco, &luna_park);

        assert!(distancia > 1170.);
        assert!(distancia < 1190.);
    }
}

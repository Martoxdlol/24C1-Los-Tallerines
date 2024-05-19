pub struct Coordenadas {
    pub lat: f64,
    pub lon: f64,
}

impl Coordenadas {
    // In metters
    pub fn distancia(&self, other: &Self) -> f64 {
        let d_lat = (self.lat - other.lat).to_radians();
        let d_lon = (self.lon - other.lon).to_radians();
        let lat1 = self.lat.to_radians();
        let lat2 = other.lat.to_radians();

        let a = (d_lat / 2.).sin().powi(2) + (d_lon / 2.).sin().powi(2) * lat1.cos() * lat2.cos();
        let c = 2. * a.sqrt().asin();

        6_371_000. * c
    }

    pub fn from_lat_lon(lat: f64, lon: f64) -> Self {
        Coordenadas { lat, lon }
    }
}

#[cfg(test)]
mod tests {
    use crate::coordenadas::Coordenadas;

    #[test]
    fn distancia_entre_obelisco_y_luna_park() {
        let obelisco = Coordenadas::from_lat_lon(-34.6037, -58.3816);
        let luna_park = Coordenadas::from_lat_lon(-34.6020, -58.3689);

        let distancia = obelisco.distancia(&luna_park);

        assert!(distancia > 1170.);
        assert!(distancia < 1190.);
    }
}

use crate::serializables::Serializable;

#[derive(Debug, Clone, Copy, PartialEq)]
/// Representa un par de coordenadas geográficas.
pub struct Coordenadas {
    pub lat: f64,
    pub lon: f64,
}

impl Coordenadas {
    /// En metros
    pub fn distancia(&self, other: &Self) -> f64 {
        let d_lat = (self.lat - other.lat).to_radians();
        let d_lon = (self.lon - other.lon).to_radians();
        let lat1 = self.lat.to_radians();
        let lat2 = other.lat.to_radians();

        let a = (d_lat / 2.).sin().powi(2) + (d_lon / 2.).sin().powi(2) * lat1.cos() * lat2.cos();
        let c = 2. * a.sqrt().asin();

        6_371_000. * c
    }

    /// El new, es la forma de crear coordenadas
    pub fn from_lat_lon(lat: f64, lon: f64) -> Self {
        Coordenadas { lat, lon }
    }

    /// Devuelve la dirección en grados de la coordenada `other` respecto a la coordenada actual.
    ///
    /// Sirve para definir la dirección del dron.
    pub fn direccion(&self, other: &Self) -> f64 {
        let diff_lat = other.lat - self.lat;
        let diff_lon = other.lon - self.lon;
        let hipotenusa = (diff_lat.powi(2) + diff_lon.powi(2)).sqrt();
        let direccion = (diff_lat / hipotenusa).acos().to_degrees();

        if diff_lon < 0. {
            360. - direccion
        } else {
            direccion
        }
    }

    /// Mueve la coordenada en una dirección y distancia dada.
    pub fn mover_en_direccion(&self, distancia: f64, direccion: f64) -> Self {
        let metros_lat = distancia * ((direccion).to_radians()).cos();
        let metros_lon = distancia * ((direccion).to_radians()).sin();

        self.mover(metros_lat, metros_lon)
    }

    /// Mueve la coordenada en una cantidad de metros dada en latitud y longitud.
    pub fn mover(&self, metros_lat: f64, metros_lon: f64) -> Self {
        let metros_por_grado = self.distancia(&Self::from_lat_lon(self.lat + 1., self.lon));
        let grados_por_metro = 1. / metros_por_grado;

        Coordenadas {
            lat: self.lat + metros_lat * grados_por_metro,
            lon: self.lon + metros_lon * grados_por_metro,
        }
    }
}

impl Serializable for Coordenadas {
    /// Serializa las coordenadas.
    fn serializar(&self) -> Vec<u8> {
        let mut serializador = crate::serializables::serializador::Serializador::new();
        serializador.agregar_elemento(&self.lat);
        serializador.agregar_elemento(&self.lon);
        serializador.bytes
    }

    /// Deserializa las coordenadas.
    fn deserializar(data: &[u8]) -> Result<Self, crate::serializables::error::DeserializationError>
    where
        Self: Sized,
    {
        let mut deserializador =
            crate::serializables::deserializador::Deserializador::new(data.to_vec());
        let lat = deserializador.sacar_elemento()?;
        let lon = deserializador.sacar_elemento()?;
        Ok(Coordenadas { lat, lon })
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

    #[test]
    fn mover() {
        let obelisco = Coordenadas::from_lat_lon(-34.6037, -58.3816);
        let destino = obelisco.mover(10000., 0.);

        println!("{:?}", destino);
        assert!(destino.lat > -34.5138);
        assert!(destino.lat < -34.5137);
        assert!(destino.lon == -58.3816);
    }

    #[test]
    fn mover_en_direccion() {
        let obelisco = Coordenadas::from_lat_lon(-34.6037, -58.3816);
        let destino = obelisco.mover_en_direccion(10000., 155.);

        println!("{:?}", destino);
        assert!(destino.lat < -34.685);
        assert!(destino.lat > -34.686);
        assert!(destino.lon < -58.343);
        assert!(destino.lon > -58.344);
    }
}

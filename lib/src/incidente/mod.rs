use crate::{
    coordenadas::Coordenadas,
    deteccion::Deteccion,
    serializables::{deserializador::Deserializador, serializador::Serializador, Serializable},
};

#[derive(Clone, Debug, PartialEq)]
/// Representa un incidente.
pub struct Incidente {
    pub id: u64,
    pub detalle: String,
    pub lat: f64,
    pub lon: f64,
    pub inicio: u64,
    pub tiempo_atendido: i64,
    pub deteccion: Option<Deteccion>,
}

impl Incidente {
    pub fn new(id: u64, detalle: String, lat: f64, lon: f64, inicio: u64) -> Self {
        Incidente {
            id,
            detalle,
            lat,
            lon,
            inicio,
            tiempo_atendido: 0,
            deteccion: None,
        }
    }

    /// Devuelve la posición del incidente.
    pub fn posicion(&self) -> Coordenadas {
        Coordenadas::from_lat_lon(self.lat, self.lon)
    }
}

impl Serializable for Incidente {
    /// Serializa el incidente.
    fn serializar(&self) -> Vec<u8> {
        let mut serializador = Serializador::new();

        serializador.agregar_elemento(&self.id);
        serializador.agregar_elemento(&self.detalle);
        serializador.agregar_elemento(&self.lat);
        serializador.agregar_elemento(&self.lon);
        serializador.agregar_elemento(&self.inicio);
        serializador.agregar_elemento(&self.tiempo_atendido);
        serializador.agregar_elemento_serializable(&self.deteccion);

        serializador.bytes
    }

    /// Deserializa el incidente.
    fn deserializar(data: &[u8]) -> Result<Self, crate::serializables::error::DeserializationError>
    where
        Self: Sized,
    {
        let mut deserializador = Deserializador::new(data.to_vec());

        let id = deserializador.sacar_elemento()?;
        let detalle = deserializador.sacar_elemento()?;
        let lat = deserializador.sacar_elemento()?;
        let lon = deserializador.sacar_elemento()?;
        let inicio = deserializador.sacar_elemento()?;
        let tiempo_atendido = deserializador.sacar_elemento()?;
        let deteccion = deserializador.sacar_elemento_serializable()?;

        Ok(Incidente {
            id,
            detalle,
            lat,
            lon,
            inicio,
            tiempo_atendido,
            deteccion,
        })
    }
}

#[cfg(test)]

mod tests {
    use crate::serializables::Serializable;

    use super::Incidente;

    #[test]
    fn serializar() {
        let incidente = Incidente::new(1, "Incidente".to_string(), 1.0, 1.0, 1);
        let serializado = incidente.serializar();
        let deserializado = Incidente::deserializar(&serializado).unwrap();

        assert_eq!(incidente, deserializado);
    }

    #[test]
    fn posicion() {
        let incidente = Incidente::new(1, "Incidente".to_string(), 1.0, 1.0, 1);
        let posicion = incidente.posicion();

        assert_eq!(posicion.lat, 1.0);
        assert_eq!(posicion.lon, 1.0);
    }
}

use crate::{
    coordenadas::Coordenadas,
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

        Ok(Incidente {
            id,
            detalle,
            lat,
            lon,
            inicio,
            tiempo_atendido,
        })
    }
}

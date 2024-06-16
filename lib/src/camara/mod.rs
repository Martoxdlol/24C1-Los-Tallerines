use std::collections::HashSet;

use crate::{
    coordenadas::Coordenadas,
    serializables::{
        deserializador::Deserializador, error::DeserializationError, serializador::Serializador,
        Serializable,
    },
};

#[derive(Debug, Clone, PartialEq)]
/// Representa una cámara de seguridad.
pub struct Camara {
    pub id: u64,
    pub lat: f64,
    pub lon: f64,
    pub rango: f64,
    /// Incidentes atendidos por cada camara.
    pub incidentes_primarios: HashSet<u64>,
    /// Incidentes que atiende cada cámara por ser lindante
    pub incidentes_secundarios: HashSet<u64>,
}

impl Camara {
    pub fn new(id: u64, lat: f64, lon: f64, rango: f64) -> Self {
        Camara {
            id,
            lat,
            lon,
            rango,
            incidentes_primarios: HashSet::new(),
            incidentes_secundarios: HashSet::new(),
        }
    }

    /// Devuelve si la cámara está activa.
    pub fn activa(&self) -> bool {
        !self.incidentes_primarios.is_empty() || !self.incidentes_secundarios.is_empty()
    }

    /// Devuelve la posición de la cámara.
    pub fn posicion(&self) -> Coordenadas {
        Coordenadas::from_lat_lon(self.lat, self.lon)
    }
}

impl Serializable for Camara {
    /// Serializa la cámara.
    fn serializar(&self) -> Vec<u8> {
        let mut serializador = Serializador::new();

        serializador.agregar_elemento(&self.id);
        serializador.agregar_elemento(&self.lat);
        serializador.agregar_elemento(&self.lon);
        serializador.agregar_elemento(&self.rango);
        serializador.agregar_elemento_serializable(&self.incidentes_primarios);
        serializador.agregar_elemento_serializable(&self.incidentes_secundarios);

        serializador.bytes
    }

    /// Deserializa la cámara.
    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError> {
        let mut deserializador = Deserializador::new(data.to_vec());

        let id = deserializador.sacar_elemento()?;
        let lat = deserializador.sacar_elemento()?;
        let lon = deserializador.sacar_elemento()?;
        let rango = deserializador.sacar_elemento()?;
        let incidentes_primarios = deserializador.sacar_elemento_serializable()?;
        let incidentes_secundarios = deserializador.sacar_elemento_serializable()?;

        Ok(Camara {
            id,
            lat,
            lon,
            rango,
            incidentes_primarios,
            incidentes_secundarios,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializar_camara() {
        let camara = Camara {
            id: 1,
            lat: 1.0,
            lon: 2.0,
            rango: 3.0,
            incidentes_primarios: vec![1, 2, 3].into_iter().collect(),
            incidentes_secundarios: vec![4, 5, 6].into_iter().collect(),
        };

        let serializado = camara.serializar();
        let deserializado = Camara::deserializar(&serializado).unwrap();

        assert_eq!(camara, deserializado);
    }

    #[test]
    fn serializar_sin_incidentes() {
        let camara = Camara {
            id: 1,
            lat: 1.0,
            lon: 2.0,
            rango: 3.0,
            incidentes_primarios: HashSet::new(),
            incidentes_secundarios: HashSet::new(),
        };

        let serializado = camara.serializar();
        let deserializado = Camara::deserializar(&serializado).unwrap();

        assert_eq!(camara, deserializado);
    }
}

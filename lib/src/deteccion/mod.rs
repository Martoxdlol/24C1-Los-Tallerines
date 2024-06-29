use std::collections::HashMap;

use crate::{
    coordenadas::Coordenadas,
    serializables::{deserializador::Deserializador, serializador::Serializador, Serializable},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Deteccion {
    id_camara: u64,
    posicion: Coordenadas,
    etiquetas: HashMap<String, f64>,
}
impl Serializable for Deteccion {
    /// Deserializa un dron.
    fn deserializar(data: &[u8]) -> Result<Self, crate::serializables::error::DeserializationError>
    where
        Self: Sized,
    {
        let mut deserializador = Deserializador::new(data.to_vec());

        let id_camara = deserializador.sacar_elemento()?;
        let posicion = deserializador.sacar_elemento_serializable()?;
        let etiquetas = deserializador.sacar_elemento_serializable()?;

        Ok(Self {
            id_camara,
            posicion,
            etiquetas,
        })
    }

    /// Serializa un dron.
    fn serializar(&self) -> Vec<u8> {
        let mut serializador = Serializador::new();

        serializador.agregar_elemento(&self.id_camara);
        serializador.agregar_elemento_serializable(&self.posicion);

        serializador.bytes
    }
}

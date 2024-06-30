use std::collections::HashMap;

use crate::{
    coordenadas::Coordenadas,
    serializables::{deserializador::Deserializador, serializador::Serializador, Serializable},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Deteccion {
    pub id_camara: u64,
    pub posicion: Coordenadas,
    pub etiquetas: HashMap<String, f64>,
}

impl Deteccion {
    pub fn es_incidente(&self) -> bool {
        self.comprobar_etiqueta("incendios", 70.) || self.comprobar_etiqueta("accidentes", 55.)
    }

    pub fn comprobar_etiqueta(&self, etiqueta: &str, confianza_minima: f64) -> bool {
        if let Some(confianza) = self.etiquetas.get(etiqueta) {
            *confianza >= confianza_minima
        } else {
            false
        }
    }

    pub fn detalle(&self) -> String {
        let mut detalle = String::new();

        for (etiqueta, confianza) in &self.etiquetas {
            detalle.push_str(&format!("{}: {:.2}% ", etiqueta, confianza));
        }

        detalle.trim_end().to_string()
    }
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
        serializador.agregar_elemento_serializable(&self.etiquetas);

        serializador.bytes
    }
}

#[cfg(test)]
mod test {
    use crate::{coordenadas::Coordenadas, deteccion::Deteccion, serializables::Serializable};
    use std::collections::HashMap;

    #[test]
    fn serializacion() {
        let mut deteccion = Deteccion {
            id_camara: 1,
            posicion: Coordenadas::from_lat_lon(1., 2.),
            etiquetas: HashMap::new(),
        };

        deteccion.etiquetas.insert("incendios".to_string(), 90.3);
        deteccion.etiquetas.insert("accidentes".to_string(), 70.2);

        let serializado = deteccion.serializar();
        let deserializado = Deteccion::deserializar(&serializado).unwrap();

        assert_eq!(deteccion, deserializado);
    }
}

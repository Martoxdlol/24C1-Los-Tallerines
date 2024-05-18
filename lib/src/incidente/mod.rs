use crate::{csv::csv_encodear_linea, serializables::Serializable};

#[derive(Clone)]
pub struct Incidente {
    pub id: u64,
    pub detalle: String,
    pub lat: f64,
    pub lon: f64,
    pub inicio: u64,
}

impl Serializable for Incidente {
    fn serializar(&self) -> Vec<u8> {
        let mut parametros: Vec<String> = Vec::new();
        parametros.push(format!("{}", self.id));
        parametros.push(self.detalle.clone());
        parametros.push(format!("{}", self.lat));
        parametros.push(format!("{}", self.lon));
        parametros.push(format!("{}", self.inicio));
        csv_encodear_linea(&parametros).into_bytes()
    }

    fn deserializar(data: &[u8]) -> Result<Self, crate::serializables::error::DeserializationError>
    where
        Self: Sized,
    {
        let linea = String::from_utf8(data.to_vec())
            .map_err(|_| crate::serializables::error::DeserializationError::InvalidData)?;
        let mut parametros = crate::csv::csv_parsear_linea(linea.as_str()).into_iter();

        let id = parametros
            .next()
            .ok_or(crate::serializables::error::DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| crate::serializables::error::DeserializationError::InvalidData)?;
        let detalle = parametros
            .next()
            .ok_or(crate::serializables::error::DeserializationError::InvalidData)?;
        let lat = parametros
            .next()
            .ok_or(crate::serializables::error::DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| crate::serializables::error::DeserializationError::InvalidData)?;
        let lon = parametros
            .next()
            .ok_or(crate::serializables::error::DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| crate::serializables::error::DeserializationError::InvalidData)?;
        let inicio = parametros
            .next()
            .ok_or(crate::serializables::error::DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| crate::serializables::error::DeserializationError::InvalidData)?;

        Ok(Incidente {
            id,
            detalle,
            lat,
            lon,
            inicio,
        })
    }
}

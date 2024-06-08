use lib::{incidente::Incidente, serializables::Serializable};

pub enum Comando {
    AtenderIncidente(Incidente),
}

impl Serializable for Comando {
    fn serializar(&self) -> Vec<u8> {
        match self {
            Comando::AtenderIncidente(incidente) => format!(
                "atender_incidente {}",
                String::from_utf8_lossy(&incidente.serializar())
            )
            .as_bytes()
            .to_vec(),
        }
    }

    fn deserializar(data: &[u8]) -> Result<Self, lib::serializables::error::DeserializationError>
    where
        Self: Sized,
    {
        let texto = String::from_utf8_lossy(data).to_string();
        let primera_palabra = texto.split(' ').next().unwrap_or("");

        if primera_palabra.eq("atender_incidente") {
            if let Ok(incidente) = Incidente::deserializar(&data[texto.len()..]) {
                return Ok(Comando::AtenderIncidente(incidente));
            }
        }

        Err(lib::serializables::error::DeserializationError::InvalidData)
    }
}

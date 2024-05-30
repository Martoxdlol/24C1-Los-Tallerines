use std::str::FromStr;
use crate::serializables::error::DeserializationError;

#[derive(Debug, Clone)]
pub enum Estado {
    EnEspera,
    VolviendoACentroDeOperacion,
    YendoAIncidente,
    AtendiendoIncidente,
    YendoACentral,
    CargandoEnCentral,
    Error,
}

impl FromStr for Estado {
    type Err = DeserializationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EnEspera" => Ok(Estado::EnEspera),
            "VolviendoACentroDeOperacion" => Ok(Estado::VolviendoACentroDeOperacion),
            "YendoAIncidente" => Ok(Estado::YendoAIncidente),
            "AtendiendoIncidente" => Ok(Estado::AtendiendoIncidente),
            "YendoACentral" => Ok(Estado::YendoACentral),
            "CargandoEnCentral" => Ok(Estado::CargandoEnCentral),
            _ => Err(DeserializationError::InvalidData),
        }
    }
}

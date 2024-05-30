use std::str::FromStr;
use crate::serializables::error::DeserializationError;

#[derive(Debug, Clone)]
pub enum EstadoDron {
    EnEspera,
    VolviendoACentroDeOperacion,
    YendoAIncidente,
    AtendiendoIncidente,
    YendoACentral,
    CargandoEnCentral,
    Error,
}

impl FromStr for EstadoDron {
    type Err = DeserializationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EnEspera" => Ok(EstadoDron::EnEspera),
            "VolviendoACentroDeOperacion" => Ok(EstadoDron::VolviendoACentroDeOperacion),
            "YendoAIncidente" => Ok(EstadoDron::YendoAIncidente),
            "AtendiendoIncidente" => Ok(EstadoDron::AtendiendoIncidente),
            "YendoACentral" => Ok(EstadoDron::YendoACentral),
            "CargandoEnCentral" => Ok(EstadoDron::CargandoEnCentral),
            _ => Err(DeserializationError::InvalidData),
        }
    }
}

use crate::incidente::Incidente;

#[derive(Debug)]
pub enum Accion {
    Incidente(Incidente),
    Cargar,
    Espera,
}

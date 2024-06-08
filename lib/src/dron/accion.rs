use crate::incidente::Incidente;

pub enum Accion {
    Incidente(Incidente),
    Cargar,
    Espera,
}

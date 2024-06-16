use crate::incidente::Incidente;

#[derive(Debug, PartialEq)]
/// Representa una acción que puede realizar un dron.
pub enum Accion {
    Incidente(Incidente),
    Cargar,
    Espera,
}

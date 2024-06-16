use crate::incidente::Incidente;

#[derive(Debug)]
/// Representa una acción que puede realizar un dron.
pub enum Accion {
    Incidente(Incidente),
    Cargar,
    Espera,
}

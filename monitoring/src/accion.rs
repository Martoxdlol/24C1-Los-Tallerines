
use crate::accion_incidente::AccionIncidente;
use crate::accion_camara::AccionCamara;
pub enum Accion {
    Incidente(AccionIncidente),
    Camara(AccionCamara),
}
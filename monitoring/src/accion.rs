use crate::accion_camara::AccionCamara;
use crate::accion_incidente::AccionIncidente;
pub enum Accion {
    Incidente(AccionIncidente),
    Camara(AccionCamara),
}

use crate::accion_camara::AccionCamara;
use crate::accion_dron::AccionDron;
use crate::accion_incidente::AccionIncidente;
pub enum AccionAplicacion {
    Incidente(AccionIncidente),
    Camara(AccionCamara),
    Dron(AccionDron),
}

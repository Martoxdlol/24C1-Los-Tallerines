use crate::accion_camara::AccionCamara;
use crate::accion_dron::AccionDron;
use crate::accion_incidente::AccionIncidente;
///Acciones de la aplicación.
///
/// Se categorizan sobre que afetan, y una vez hecho se dividen para cada uno.
pub enum AccionAplicacion {
    Incidente(AccionIncidente),
    Camara(AccionCamara),
    Dron(AccionDron),
}

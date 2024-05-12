use crate::camara::id::IdCamara;

pub enum Comando {
    Conectar(IdCamara, f64, f64, f64),
    Desconectar(IdCamara),
    ListarCamaras,
    ModificarUbicacion(IdCamara, f64, f64),
    ModifciarRango(IdCamara, f64),
    Ayuda,
    
}
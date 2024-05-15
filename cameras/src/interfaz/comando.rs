use crate::camara::id::IdCamara;

/// Comandos que podemos pedirle al sistema de cámaras vía terminal.
pub enum Comando {
    Conectar(IdCamara, f64, f64, f64),
    Desconectar(IdCamara),
    ListarCamaras,
    MostrarCamara(IdCamara),
    ModificarUbicacion(IdCamara, f64, f64),
    ModifciarRango(IdCamara, f64),
    Ayuda,
}

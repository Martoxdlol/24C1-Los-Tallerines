/// Comandos que podemos pedirle al sistema de cámaras vía terminal.
pub enum Comando {
    Conectar(u64, f64, f64, f64),
    Desconectar(u64),
    ListarCamaras,
    Camara(u64),
    ModificarUbicacion(u64, f64, f64),
    ModifciarRango(u64, f64),
    Ayuda,
}

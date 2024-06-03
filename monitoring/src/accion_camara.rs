pub enum AccionCamara {
    Conectar,
    Modificar(u64),
    CambiarRango(u64),
    CambiarUbicacion(u64),
}
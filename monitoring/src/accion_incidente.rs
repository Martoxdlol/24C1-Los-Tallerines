/// Enum para la ventana de la esquina superior izquierda.
pub enum AccionIncidente {
    Crear,
    Modificar(u64),
    CambiarDetalle(u64),
    CambiarUbicacion(u64),
}

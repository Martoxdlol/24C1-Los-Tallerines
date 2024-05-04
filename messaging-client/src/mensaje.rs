use crate::publicacion::Publicacion;

pub enum Mensaje {
    Publicacion(String, Publicacion),
    Info,
    Ping,
    Pong,
}

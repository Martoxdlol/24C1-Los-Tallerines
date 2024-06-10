use crate::publicacion::mensaje::PublicacionMensaje;

use super::tick_contexto::TickContexto;

pub trait Conexion {
    fn tick(&mut self, salida: &mut TickContexto);

    fn escribir_publicacion_mensaje(&mut self, mensaje: &PublicacionMensaje);

    fn esta_conectado(&self) -> bool;
}

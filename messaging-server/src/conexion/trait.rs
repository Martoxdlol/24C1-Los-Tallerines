use crate::publicacion::mensaje::PublicacionMensaje;

use super::tick_contexto::TickContexto;

pub trait Conexion {
    fn obtener_id(&self) -> u64;

    fn tick(&mut self, salida: &mut TickContexto);

    fn escribir_publicacion_mensaje(&mut self, mensaje: &PublicacionMensaje);

    fn esta_conectado(&self) -> bool;

    fn setear_id_conexion(&mut self, id_conexion: u64);
}

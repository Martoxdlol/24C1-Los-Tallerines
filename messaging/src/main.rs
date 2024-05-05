use servidor::Servidor;

pub mod conexion;
pub mod message;
pub mod parser;
pub mod proceso;
pub mod publicacion;
pub mod respuesta;
pub mod resultado_linea;
pub mod servidor;
pub mod topico;
pub mod configuracion;
pub mod stream;
pub mod instrucciones;
pub mod publicacion_mensaje;

// mod tests;

fn main() {
    let mut servidor = Servidor::procesos(4);

    servidor.inicio();
}

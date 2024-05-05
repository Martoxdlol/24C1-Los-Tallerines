use servidor::Servidor;

pub mod conexion;
pub mod configuracion;
pub mod message;
pub mod parser;
pub mod proceso;
pub mod publicacion;
pub mod respuesta;
pub mod resultado_linea;
pub mod servidor;
pub mod stream;
pub mod subject;

fn main() {
    let mut servidor = Servidor::procesos(4);

    servidor.inicio();
}

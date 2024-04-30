use servidor::Servidor;

pub mod conexion;
pub mod message;
pub mod parser;
pub mod proceso;
pub mod publicacion;
pub mod respuesta;
pub mod resultado_linea;
pub mod servidor;
pub mod subject;
pub mod configuracion;

fn main() {
    let mut servidor = Servidor::procesos(4);

    servidor.inicio();
}

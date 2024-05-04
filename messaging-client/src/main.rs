use client::Cliente;

pub mod client;
pub mod hilo_cliente;
pub mod instruccion;
pub mod mensaje;
pub mod publicacion;
pub mod subscripcion;

fn main() {
    let mut cliente = Cliente::conectar("localhost:4222").unwrap();

    let sub = cliente.subscribe("asd", None).unwrap();

    cliente.publicar("asd", b"hola", None).unwrap();

    loop {}
}

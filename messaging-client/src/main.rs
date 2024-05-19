use messaging_client::cliente::Cliente;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut cliente = Cliente::conectar("localhost:4222")?;

    let mut sub = cliente.suscribirse("asd", None)?;

    cliente.publicar("asd", b"hola", None)?;

    if let Ok(publicacion) = sub.leer() {
        println!("{:?}", publicacion);
    }

    drop(sub);

    drop(cliente);

    //loop {}

    Ok(())
}

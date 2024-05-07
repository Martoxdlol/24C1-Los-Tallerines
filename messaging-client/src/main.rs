use cliente::Cliente;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut cliente = Cliente::conectar("localhost:4222")?;

    let sub = cliente.suscribirse("asd", None)?;

    //cliente.publicar("asd", b"hola", None)?;

    // for msg in sub {}

    //  match sub.try_recv() {
    //     Ok(msg) => {
    //         println!("Mensaje: {:?}", msg);
    //     }
    //  }

    drop(sub);

    drop(cliente);

    //loop {}

    Ok(())
}

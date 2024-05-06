

// mod tests;

use messaging::servidor::Servidor;

fn main() {
    let mut servidor = Servidor::procesos(4);

    servidor.inicio();
}

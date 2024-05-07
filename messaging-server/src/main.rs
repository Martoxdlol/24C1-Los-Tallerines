use messaging_server::servidor::Servidor;

fn main() {
    let mut servidor = Servidor::procesos(4);

    servidor.inicio();
}

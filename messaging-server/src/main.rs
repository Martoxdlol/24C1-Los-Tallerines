use messaging_server::servidor::Servidor;

fn main() {
    // Cantidad de hilos que se van a crear
    let mut servidor = Servidor::procesos(4);

    servidor.inicio();
}

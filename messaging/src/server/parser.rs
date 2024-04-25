use super::message::Message;

pub struct Parser {
    bytes: Vec<u8>,
    ultimo_byte_procesado: usize,
}

/// La responsabilidad del parser es recibir bytes de la conexión y tranformarlos a mensajes
///
/// Acumula los bytes que recibe y cuando tiene suficientes bytes para formar un mensaje
/// lo parsea y lo devuelve
///
/// El proceso de utilizar los bytes que fue recibiendo y convertirlos a mensajes se llama
/// desde la función `proximo_mensaje`, esta te entrega mensajes uno por uno
///
/// El parser se encarga de ir liberando los bytes que ya utilizó y de tener en cuenta los estados intermedios
/// (es decir, si le llega un mensaje que no está completo, lo guarda y espera a que lleguen más bytes)
impl Parser {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            ultimo_byte_procesado: 0,
        }
    }

    pub fn agregar_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub fn proximo_mensaje(&self) -> Option<Message> {
        // Parsear los bytes que se leyeron
        // y hacer algo con ellos
        None
    }

    /// Si se encontro un mensaje en proximo_mensaje, se deben liberar los bytes del mismo.
    fn resetear(&mut self) {
        self.ultimo_byte_procesado = 0;
        self.bytes.clear();
    }
}

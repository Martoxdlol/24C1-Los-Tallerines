#[derive(Debug, PartialEq)]
pub enum ResultadoLinea {
    StringVacio,
    MensajeIncorrecto,
    Pub(String, Option<String>, usize),
    Hpub(String, Option<String>, usize, usize),
    Sub(String, Option<String>, String),
    Unsub(String, Option<u64>),
    Ping,
    Connect,
}

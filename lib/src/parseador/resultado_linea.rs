#[derive(Debug, PartialEq)]
pub enum ResultadoLinea {
    StringVacio,
    MensajeIncorrecto,
    Pub(String, Option<String>, usize),
    Hpub(String, Option<String>, usize, usize),
    Msg(String, String, Option<String>, usize),
    Hmsg(String, String, Option<String>, usize, usize),
    Sub(String, Option<String>, String),
    Unsub(String, Option<u64>),
    Ping,
    Pong,
    Info,
    Connect,
    Ok,
    Err,
}

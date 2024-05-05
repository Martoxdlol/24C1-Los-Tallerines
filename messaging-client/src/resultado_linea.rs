#[derive(Debug)]
pub enum ResultadoLinea {
    StringVacio,
    MensajeIncorrecto,
    Msg {
        subject: String,
        sid: String,
        reply_to: Option<String>,
        payload_bytes: usize,
    },
    HMsg {
        subject: String,
        sid: String,
        reply_to: Option<String>,
        payload_bytes: usize,
        header_bytes: usize,
    },
    Info,
    Ping,
    Pong,
}

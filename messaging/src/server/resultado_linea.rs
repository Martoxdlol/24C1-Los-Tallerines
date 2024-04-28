#[derive(Debug)]
pub enum ResultadoLinea {
    StringVacio,
    SintaxisInvalida,
    Pub(String, Option<String>, usize),
    Hpub(String, Option<String>, usize, usize),
    Sub(String, Option<String>, String),
    Unsub(String, Option<usize>),
}

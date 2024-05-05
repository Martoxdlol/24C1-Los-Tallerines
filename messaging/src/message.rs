#[derive(Debug)]
pub enum Message {
    // 'topico', 'replay_to' payload
    Pub(String, Option<String>, Vec<u8>),
    // 'topico', 'replay_to' headers, payload
    Hpub(String, Option<String>, Vec<u8>, Vec<u8>),
    // 'topico', 'queue group', 'id
    Sub(String, Option<String>, String),
    //
    Unsub(String, Option<u64>),
    // Mensaje de error (cuando no se pudo parsear el mensaje)
    Err(String),
    // Mensaje para generar la conexión
    Connect(String),
    // Mensaje para revisar la conexión
    Ping()
}

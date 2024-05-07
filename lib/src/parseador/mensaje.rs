#[derive(Debug)]
pub enum Mensaje {
    // 'topico', 'replay_to' payload
    Publicar(String, Option<String>, Vec<u8>),
    // 'topico', 'replay_to' headers, payload
    PublicarConHeader(String, Option<String>, Vec<u8>, Vec<u8>),
    // 'topico', 'queue group', 'id
    Suscribir(String, Option<String>, String),
    //
    Desuscribir(String, Option<u64>),
    // Mensaje de error (cuando no se pudo parsear el mensaje)
    Error(String),
    // Mensaje para generar la conexión
    Conectar(String),
    // Mensaje para revisar la conexión
    Ping(),
}

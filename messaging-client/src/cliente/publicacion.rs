/// Estructura de una publicación (Pub)
#[derive(Debug)]
pub struct Publicacion {
    pub subject: String,
    pub replay_to: Option<String>,
    pub payload: Vec<u8>,
    pub header: Option<Vec<u8>>,
}
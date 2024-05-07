use crate::{publicacion::Publicacion, suscripciones::suscripcion::Suscripcion};

/// Las instrucciones se envian entre "threads" o hilos
#[derive(Clone, Debug)]
pub enum Instrucciones {
    /// Añadir una suscripción
    Suscribir(Suscripcion),
    /// Eliminar una suscripción
    Desuscribir(Suscripcion),
    /// Publicar, excepto suscripciones de queue group
    Publicar(Publicacion),
    /// Enviar una publicación a una suscripción exacta
    PublicarExacto(Suscripcion, Publicacion),
}

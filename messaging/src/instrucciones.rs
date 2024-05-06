use crate::{publicacion::Publicacion, topico::Topico};

/// Las instrucciones se envian entre "threads" o hilos
#[derive(Clone, Debug)]
pub enum Instrucciones {
    /// Añadir una suscripción a un topico
    Suscribir(u64, Topico),
    /// Eliminar una suscripción a un topico
    Desuscribir(u64, Topico),
    /// Colas o "Queue Groups". Cada thread puede suscribir a una cola,
    /// y cada suscripción tiene un peso según cuantas conexiones tiene internamente suscritas a esa cola.
    /// Esta instrucción permite actualizar el peso de una cola para una thread.
    ActualizarCola {
        id: u64,
        cola: String,
        topico: Topico,
        peso: u64,
    },
    /// Publicar, excepto suscripciones de queue group
    Publicar(Publicacion),
    /// Publicar a una cola o "Queue Group"
    PublicarCola(String, Publicacion),
}

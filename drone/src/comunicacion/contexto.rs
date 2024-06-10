use messaging_client::cliente::{suscripcion::Suscripcion, Cliente};

pub struct Contexto {
    pub cliente: Cliente,
    pub suscripcion_incidentes_creados: Suscripcion,
    pub suscripcion_incidentes_finalizados: Suscripcion,
    pub suscripcion_comandos: Suscripcion,
}

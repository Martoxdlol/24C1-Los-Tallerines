use messaging_client::cliente::{
    jetstream::js_suscripcion::JSSuscripcion, suscripcion::Suscripcion, Cliente,
};

pub struct Contexto {
    pub cliente: Cliente,
    pub suscripcion_incidentes_finalizados: Suscripcion,
    pub suscripcion_comandos: JSSuscripcion,
}

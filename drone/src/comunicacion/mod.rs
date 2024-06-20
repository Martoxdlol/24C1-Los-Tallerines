pub mod comando;
pub mod contexto;

use std::io;

use comando::Comando;
use contexto::Contexto;
use lib::{
    configuracion::Configuracion, dron::Dron, incidente::Incidente, serializables::Serializable,
};
use messaging_client::cliente::Cliente;

/// Comunicación desde el dron con el servidor de mensajería.
pub struct Comunicacion {
    direccion_server: String,
    puerto_server: u16,
    user: Option<String>,
    pass: Option<String>,
    contexto: Option<Contexto>,
}

impl Comunicacion {
    pub fn new(config: &Configuracion) -> Self {
        Self {
            direccion_server: config
                .obtener::<String>("direccion")
                .unwrap_or("127.0.0.1".to_string()),
            puerto_server: config.obtener::<u16>("puerto").unwrap_or(4222),
            user: config.obtener::<String>("user"),
            pass: config.obtener::<String>("pass"),
            contexto: None,
        }
    }

    /// Intenta usar el contexto actual, si no existe, crea uno nuevo.
    pub fn usar_contexto(&mut self, dron: &Dron) -> io::Result<&Contexto> {
        if self.contexto.is_none() {
            let mut cliente = Cliente::conectar_user_pass(
                format!("{}:{}", self.direccion_server, self.puerto_server,).as_str(),
                self.user.clone(),
                self.pass.clone(),
            )?;

            let suscripcion_incidentes_finalizados =
                cliente.suscribirse("incidentes.*.finalizado", None)?;
            let suscripcion_comandos =
                cliente.suscribirse(format!("drones.{}.comandos", dron.id).as_str(), None)?;

            self.contexto = Some(Contexto {
                cliente,
                suscripcion_incidentes_finalizados,
                suscripcion_comandos,
            });
        }

        Ok(self.contexto.as_mut().unwrap())
    }

    /// Ciclo de comunicación del dron con el servidor de mensajería.
    pub fn ciclo(&mut self, drone: &mut Dron) {
        if let Err(e) = self.ciclo_interno(drone) {
            eprintln!("Error en ciclo interno: {}", e);
            self.contexto = None;
        }
    }

    /// Ciclo interno de comunicación del dron con el servidor de mensajería.
    ///
    /// Se encarga de enviar el estado del dron, recibir comandos y recibir incidentes finalizados.
    fn ciclo_interno(&mut self, dron: &mut Dron) -> io::Result<()> {
        let mut tiempo = 1500;

        if dron.velocidad_actual > 0. {
            tiempo = 300
        }

        if chrono::offset::Local::now().timestamp_millis() - dron.envio_ultimo_estado > tiempo {
            self.enviar_estado(dron)?;
        }

        self.recibir_comandos(dron)?;
        self.recibir_incidentes_finalizados(dron)?;

        Ok(())
    }

    /// envia el estado del dron al servidor de mensajería.
    fn enviar_estado(&mut self, dron: &mut Dron) -> io::Result<()> {
        println!(
            "{:?} bateria: {}, velocidad: {}, destino: {:?}, acción: {:?}",
            dron.posicion,
            dron.bateria_actual,
            dron.velocidad_actual,
            dron.destino(),
            dron.accion()
        );

        let contexto = self.usar_contexto(dron)?;
        contexto
            .cliente
            .publicar(&format!("drones.{}", dron.id), &dron.serializar(), None)?;

        dron.envio_ultimo_estado = chrono::offset::Local::now().timestamp_millis();

        Ok(())
    }

    /// Recibe comandos del servidor de mensajería.
    ///
    /// Estos pueden ser llamados a atender incidentes o desatender incidentes.
    fn recibir_comandos(&mut self, dron: &mut Dron) -> io::Result<()> {
        let contexto = self.usar_contexto(dron)?;

        let mut enviar_estado = false;

        while let Some(publicacion) = contexto.suscripcion_comandos.intentar_leer()? {
            if let Ok(comando) = Comando::deserializar(&publicacion.payload) {
                println!("Comando: {:?}", comando);

                match comando {
                    Comando::AtenderIncidente(incidente) => {
                        if dron.incidente_actual.is_some() {
                            continue;
                        }

                        dron.incidente_actual = Some(incidente);
                    }
                    Comando::DesatenderIncidente(incidente) => {
                        if let Some(incidente_dron) = &dron.incidente_actual {
                            if incidente_dron.id.eq(&incidente.id) {
                                dron.incidente_actual = None;
                            }
                        }
                    }
                }

                enviar_estado = true;
            }
        }

        if enviar_estado {
            self.enviar_estado(dron)?;
        }

        Ok(())
    }

    /// Se encarga de recibir los incidentes finalizados.
    ///
    /// Si el incidente finalizado es el mismo que el incidente actual del dron, se desasigna el incidente actual.
    fn recibir_incidentes_finalizados(&mut self, dron: &mut Dron) -> io::Result<()> {
        let contexto = self.usar_contexto(dron)?;
        while let Some(publicacion) = contexto
            .suscripcion_incidentes_finalizados
            .intentar_leer()?
        {
            if let Ok(incidente_finalizado) = Incidente::deserializar(&publicacion.payload) {
                if let Some(incidente_dron) = &dron.incidente_actual {
                    if incidente_finalizado.id.eq(&incidente_dron.id) {
                        dron.incidente_actual = None;
                    }
                }
            }
        }

        Ok(())
    }
}

pub mod comando;
pub mod contexto;

use std::io;

use comando::Comando;
use contexto::Contexto;
use lib::{
    configuracion::Configuracion, dron::Dron, incidente::Incidente, serializables::Serializable,
};
use messaging_client::cliente::Cliente;

pub struct Comunicacion {
    direccion_server: String,
    puerto_server: u16,
    user: Option<String>,
    pass: Option<String>,
    ultimo_envio_estado: i64,
    contexto: Option<Contexto>,
}

impl Comunicacion {
    pub fn new(config: &Configuracion) -> Self {
        Self {
            ultimo_envio_estado: 0,
            direccion_server: config
                .obtener::<String>("direccion_server")
                .unwrap_or("127.0.0.1".to_string()),
            puerto_server: config.obtener::<u16>("puerto_server").unwrap_or(4222),
            user: config.obtener::<String>("user"),
            pass: config.obtener::<String>("pass"),
            contexto: None,
        }
    }

    pub fn usar_contexto(&mut self, dron: &Dron) -> io::Result<&Contexto> {
        if self.contexto.is_none() {
            let mut cliente = Cliente::conectar_user_pass(
                format!("{}:{}", self.direccion_server, self.puerto_server,).as_str(),
                self.user.clone(),
                self.pass.clone(),
            )?;

            let suscripcion_incidentes_creados =
                cliente.suscribirse("incidentes.*.creado", None)?;
            let suscripcion_incidentes_finalizados =
                cliente.suscribirse("incidentes.*.finalizado", None)?;
            let suscripcion_comandos =
                cliente.suscribirse(format!("drones.{}.comandos", dron.id).as_str(), None)?;

            self.contexto = Some(Contexto {
                cliente,
                suscripcion_incidentes_creados,
                suscripcion_incidentes_finalizados,
                suscripcion_comandos,
            });
        }

        Ok(self.contexto.as_mut().unwrap())
    }

    pub fn ciclo(&mut self, drone: &mut Dron) {
        if let Err(e) = self.ciclo_interno(drone) {
            eprintln!("Error en ciclo interno: {}", e);
            self.contexto = None;
        }
    }

    fn ciclo_interno(&mut self, dron: &mut Dron) -> io::Result<()> {
        if chrono::offset::Local::now().timestamp_millis() - self.ultimo_envio_estado > 1000 {
            println!(
                "{}",
                chrono::offset::Local::now().timestamp_millis() - self.ultimo_envio_estado
            );
            self.enviar_estado(dron)?;
        }

        self.recibir_incidentes(dron)?;
        self.recibir_comandos(dron)?;
        self.recibir_incidentes_finalizados(dron)?;

        Ok(())
    }

    fn enviar_estado(&mut self, dron: &mut Dron) -> io::Result<()> {
        let contexto = self.usar_contexto(dron)?;
        contexto
            .cliente
            .publicar(&format!("drones.{}", dron.id), &dron.serializar(), None)?;

        self.ultimo_envio_estado = chrono::offset::Local::now().timestamp_millis();

        Ok(())
    }

    fn recibir_comandos(&mut self, dron: &mut Dron) -> io::Result<()> {
        let contexto = self.usar_contexto(dron)?;

        while let Some(publicacion) = contexto.suscripcion_comandos.intentar_leer()? {
            if let Ok(comando) = Comando::deserializar(&publicacion.payload) {
                match comando {
                    Comando::AtenderIncidente(incidente) => {
                        dron.incidente_actual = Some(incidente);
                    }
                }
            }
        }

        Ok(())
    }

    fn recibir_incidentes(&mut self, dron: &mut Dron) -> io::Result<()> {
        let contexto = self.usar_contexto(dron)?;

        while let Some(publicacion) = contexto.suscripcion_incidentes_creados.intentar_leer()? {
            match dron.accion() {
                lib::dron::accion::Accion::Cargar => {
                    continue;
                }
                lib::dron::accion::Accion::Incidente(_) => {
                    continue;
                }
                _ => {}
            }

            if let Ok(incidente) = Incidente::deserializar(&publicacion.payload) {
                if incidente.posicion().distancia(&dron.posicion) < dron.rango {
                    contexto.cliente.publicar(
                        &format!("incidentes.{}.dron", incidente.id),
                        &dron.serializar(),
                        None,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn recibir_incidentes_finalizados(&mut self, dron: &mut Dron) -> io::Result<()> {
        let contexto = self.usar_contexto(dron)?;
        while let Some(publicacion) = contexto
            .suscripcion_incidentes_finalizados
            .intentar_leer()?
        {
            if let Ok(incidente_finalizado) = Incidente::deserializar(&publicacion.payload) {
                if let Some(incidente_dron) = dron.incidente_actual.take() {
                    if incidente_finalizado.id.eq(&incidente_dron.id) {
                        dron.incidente_actual = None;
                    }
                }
            }
        }

        Ok(())
    }
}

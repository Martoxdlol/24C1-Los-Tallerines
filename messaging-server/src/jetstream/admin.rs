use std::sync::mpsc::Sender;

use crate::{
    conexion::{r#trait::Conexion, tick_contexto::TickContexto},
    publicacion::Publicacion,
    suscripciones::{suscripcion::Suscripcion, topico::Topico},
};

pub struct JestStreamAdminConexion {
    id: u64,
    preparado: bool,
    tx_conexiones: Sender<Box<dyn Conexion + Send>>,
    respuestas: Vec<Publicacion>,
}

impl JestStreamAdminConexion {
    pub fn new(
        id: u64,
        tx_conexiones: Sender<Box<dyn Conexion + Send>>,
    ) -> JestStreamAdminConexion {
        JestStreamAdminConexion {
            preparado: false,
            id,
            tx_conexiones,
            respuestas: Vec::new(),
        }
    }
}

impl Conexion for JestStreamAdminConexion {
    fn obtener_id(&self) -> u64 {
        self.id
    }

    fn tick(&mut self, contexto: &mut TickContexto) {
        if !self.preparado {
            contexto.suscribir(Suscripcion::new(
                contexto.id_hilo,
                self.id,
                // $JS.API.STREAM.CREATE.incidentes
                Topico::new("$JS.API.STREAM.CREATE.*".to_string()).unwrap(),
                "stream.crear".to_string(),
                None,
            ));
            self.preparado = true;
        }
    }

    fn escribir_publicacion_mensaje(
        &mut self,
        mensaje: &crate::publicacion::mensaje::PublicacionMensaje,
    ) {
        match mensaje.sid.as_str() {
            "stream.crear" => {
                println!("JestStreamHilo::escribir_publicacion_mensaje: stream.crear");
            }
            _ => {}
        }
    }

    fn esta_conectado(&self) -> bool {
        true
    }
}

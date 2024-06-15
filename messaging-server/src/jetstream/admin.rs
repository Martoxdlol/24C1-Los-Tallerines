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

    fn suscribir(&self, contexto: &mut TickContexto, topico: &str, sid: &str) {
        contexto.suscribir(Suscripcion::new(
            contexto.id_hilo,
            self.id,
            Topico::new(topico.to_string()).unwrap(),
            "".to_string(),
            None,
        ));
    }
}

impl Conexion for JestStreamAdminConexion {
    fn obtener_id(&self) -> u64 {
        self.id
    }

    fn tick(&mut self, contexto: &mut TickContexto) {
        if !self.preparado {
            self.suscribir(contexto, "$JS.API.STREAM.CREATE.*", "stream.crear");
            self.suscribir(contexto, "$JS.API.STREAM.LIST", "stream.listar");
            self.suscribir(contexto, "$JS.API.STREAM.NAMES", "stream.nombres");

            // SE SUSCRIBE EL STREAM QUE CORRESPONA
            // self.suscribir(contexto, "$JS.API.STREAM.INFO.<nombre stream>", "stream.info");
            // self.suscribir(contexto, "$JS.API.STREAM.DELETE.<nombre stream>", "stream.eliminar");
            // self.suscribir(contexto, "$JS.API.STREAM.UPDATE.<nombre stream>", "stream.actualizar");
            // self.suscribir(contexto, "$JS.API.STREAM.PURGE.<nombre stream>", "stream.purgar");
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

                // self.tx_conexiones.send(JetStreamStream::new(nombre))
            }
            "stream.listar" => {
                println!("JestStreamHilo::escribir_publicacion_mensaje: stream.listar");
            }
            "stream.nombres" => {
                println!("JestStreamHilo::escribir_publicacion_mensaje: stream.nombres");
            }
            _ => {}
        }
    }

    fn esta_conectado(&self) -> bool {
        true
    }
}

use std::{
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

use super::{conexion::Conexion, publicacion::Publicacion};

pub struct Proceso {
    /// Canales a otros procesos
    otros_procesos: Vec<Sender<Publicacion>>, // El tipo sender es un canal de solo escritura
    nuevos_mensajes: Receiver<Publicacion>, // El tipo receiver es un canal de solo lectura
    nuevas_conexiones: Receiver<Conexion>,
    conexiones: Vec<Conexion>,
}

impl Proceso {
    pub fn new(
        otros_procesos: Vec<Sender<Publicacion>>,
        canal: Receiver<Publicacion>,
        nuevas_conexiones: Receiver<Conexion>,
    ) -> Proceso {
        Proceso {
            otros_procesos,
            nuevos_mensajes: canal,
            nuevas_conexiones,
            conexiones: Vec::new(),
        }
    }

    pub fn iniciar(mut proceso: Proceso) -> JoinHandle<()> {
        thread::spawn(move || { // Crea un nuevo hilo y ejecuta proceso.inicio() dentro
            proceso.inicio();
        })
    }

    fn inicio(&mut self) {
        loop {
            // Recibir nuevas conexiones iniciadas al servidor
            while let Ok(conexion) = self.nuevas_conexiones.try_recv() {
                self.conexiones.push(conexion);
            }

            // Todas las publicaciones que se recibieron en esta iteración
            // Generadas por las conexiones de este proceso
            // Y por las conexiones de los otros procesos
            let mut todas_las_publicaciones = Vec::new();

            // Por cada conexión,
            for conexion in &mut self.conexiones {
                // Ejecutar el tick (cada tick recibe, procesa y envía mensajes pendientes)
                conexion.tick();

                // Extraer las publicaciones salientes que se pueden haber generado en el tick
                let publicaciones_salientes = conexion.extraer_publicaciones_salientes();

                // Agregar las publicaciones salientes al vector de publicaciones
                todas_las_publicaciones.extend(publicaciones_salientes);
            }

            // Enviamos todas las publicaciones creadas por este proceso a los otros procesos
            for tx in self.otros_procesos.iter() {
                for publicacion in &todas_las_publicaciones {
                    let r = tx.send(publicacion.clone());
                    if r.is_err() {
                        panic!("No se pudo enviar la publicación a otro proceso");
                    }
                }
            }

            // Recibir nuevas publicaciones de otros procesos
            while let Ok(publicacion) = self.nuevos_mensajes.try_recv() {
                todas_las_publicaciones.push(publicacion);
            }

            // Por cada conexión y por cada cliente, enviarle todas las publicaciones
            // Cada conexión decide si debe enviar el mensaje o ignorarlo según el tópico
            for conexion in &mut self.conexiones {
                for publicacion in &todas_las_publicaciones {
                    conexion.recibir_mensaje(publicacion.clone());
                }
            }

            self.conexiones.retain(|conexion| !conexion.desconectado);
        }
    }
}

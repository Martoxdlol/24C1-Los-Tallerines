pub mod id;

use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

use crate::{
    conexion::{id::IdConexion, tick_contexto::TickContexto, Conexion},
    instrucciones::Instrucciones,
    publicacion::Publicacion,
    registrador::Registrador,
    suscripciones::{suscripcion::Suscripcion, Suscripciones},
};

use self::id::IdHilo;

pub struct Hilo {
    id: u64,
    /// Canal para **recibir** instrucciones de otros procesos
    canal_recibir_conexiones: Receiver<(IdConexion, Conexion)>,
    /// Canales a otros hilos para **enviar** instrucciones (ejemplo: publicar, suscribir, desuscribir, etc.)
    canales_enviar_instrucciones: HashMap<IdHilo, Sender<Instrucciones>>,
    /// Canal para **recibir** instrucciones de otros procesos
    canal_recibir_instrucciones: Receiver<Instrucciones>,
    /// Suscripciones de este hilo
    suscripciones: Suscripciones,
    /// Retgisrador de eventos
    registrador: Registrador,
    /// Conexiones de este hilo
    conexiones: HashMap<IdConexion, Conexion>,
}

impl Hilo {
    pub fn new(
        id: u64,
        canal_recibir_conexiones: Receiver<(IdConexion, Conexion)>,
        canales_enviar_instrucciones: HashMap<IdHilo, Sender<Instrucciones>>,
        canal_recibir_instrucciones: Receiver<Instrucciones>,
        registrador: Registrador,
    ) -> Self {
        Self {
            id,
            canal_recibir_conexiones,
            canales_enviar_instrucciones,
            canal_recibir_instrucciones,
            registrador,
            suscripciones: Suscripciones::new(),
            conexiones: HashMap::new(),
        }
    }

    /// Inicial la ejecución del hilo
    pub fn iniciar(mut hilo: Hilo) -> JoinHandle<()> {
        thread::spawn(move || {
            // Crea un nuevo hilo y ejecuta proceso.inicio() dentro
            hilo.inicio();
        })
    }

    /// Punto inicial de ejecución del hilo, este nunca termina
    /// (al menos que ocurra un error fatal).
    pub fn inicio(&mut self) {
        loop {
            self.tick();
        }
    }

    /// Este método se ejecuta en cada ciclo del hilo.
    /// Se encarga de procesar las instrucciones recibidas y
    /// realizar las acciones correspondientes.
    pub fn tick(&mut self) {
        self.recibir_conexiones();
        self.recibir_instrucciones();
    }

    pub fn recibir_conexiones(&mut self) {
        while let Ok((id_conexion, conexion)) = self.canal_recibir_conexiones.try_recv() {
            self.registrador
                .info(&format!("Recibida conexión con id {}", id_conexion), None);
            self.conexiones.insert(id_conexion, conexion);
        }
    }

    pub fn recibir_instrucciones(&mut self) {
        while let Ok(instruccion) = self.canal_recibir_instrucciones.try_recv() {
            println!(
                "[hilo: {}] Recibida instrucción: {:?}",
                self.id, &instruccion
            );
            self.recibir_instruccion(instruccion);
        }
    }

    pub fn recibir_instruccion(&mut self, instruccion: Instrucciones) {
        match instruccion {
            Instrucciones::Suscribir(suscripcion) => {
                self.suscripciones.suscribir(suscripcion);
            }
            Instrucciones::Desuscribir(suscripcion) => {
                self.suscripciones.desuscribir(&suscripcion);
            }
            Instrucciones::Publicar(publicacion) => {
                self.recibir_publicacion(publicacion);
            }
            Instrucciones::PublicarExacto(suscripcion, publicacion) => {
                self.recibir_publicacion_exacto(&suscripcion, publicacion);
            }
        }
    }

    pub fn recibir_publicacion(&mut self, publicacion: Publicacion) {
        // Iterar sobre las suscripciones y enviar la publicación a cada una
        // Cabe destacar que solo itera en las suscripciones que coinciden con el tópico de la publicación
        self.suscripciones
            .visitar_suscripciones_por_topico(&publicacion.topico, |suscripcion| {
                // Enviar la publicación a la conexión
                if let Some(conexion) = self.conexiones.get_mut(&suscripcion.id_conexion()) {
                    conexion.escribir_publicacion_mensaje(
                        &publicacion.mensaje(suscripcion.id().to_owned()),
                    );
                }
            })
    }

    pub fn recibir_publicacion_exacto(
        &mut self,
        suscripcion: &Suscripcion,
        publicacion: Publicacion,
    ) {
        if let Some(conexion) = self.conexiones.get_mut(&suscripcion.id_conexion()) {
            conexion
                .escribir_publicacion_mensaje(&publicacion.mensaje(suscripcion.id().to_owned()));
        }
    }

    pub fn tick_conexiones(&mut self) {
        let mut salidas = Vec::new();

        for conexion in self.conexiones.values_mut() {
            let mut tick_salida = TickContexto::new(self.id);
            conexion.tick(&mut tick_salida);
            salidas.push(tick_salida);
        }

        for salida in salidas {
            for suscripcion in salida.suscripciones {
                self.suscripciones.suscribir(suscripcion);
            }

            for suscripcion in salida.desuscripciones {
                todo!();
                // self.suscripciones.desuscribir(&suscripcion);
            }

            for publicacion in salida.publicaciones {
                self.recibir_publicacion(publicacion);
            }
        }
    }
}

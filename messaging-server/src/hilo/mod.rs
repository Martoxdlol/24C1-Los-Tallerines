pub mod id;
pub mod instruccion;

use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

use crate::{
    conexion::{id::IdConexion, r#trait::Conexion, tick_contexto::TickContexto},
    publicacion::Publicacion,
    registrador::Registrador,
    suscripciones::{suscripcion::Suscripcion, Suscripciones},
};

use self::{id::IdHilo, instruccion::Instruccion};

pub struct Hilo {
    id: u64,
    /// Canal para **recibir** instrucciones de otros procesos
    canal_recibir_conexiones: Receiver<(IdConexion, Box<dyn Conexion + Send>)>,
    /// Canales a otros hilos para **enviar** instrucciones (ejemplo: publicar, suscribir, desuscribir, etc.)
    canales_enviar_instrucciones: HashMap<IdHilo, Sender<Instruccion>>,
    /// Canal para **recibir** instrucciones de otros procesos
    canal_recibir_instrucciones: Receiver<Instruccion>,
    /// Suscripciones de este hilo
    suscripciones: Suscripciones,
    /// Registrador de eventos
    registrador: Registrador,
    /// Conexiones de este hilo
    conexiones: HashMap<IdConexion, Box<dyn Conexion + Send>>,
}

impl Hilo {
    pub fn new(
        id: u64,
        canal_recibir_conexiones: Receiver<(IdConexion, Box<dyn Conexion + Send>)>,
        canales_enviar_instrucciones: HashMap<IdHilo, Sender<Instruccion>>,
        canal_recibir_instrucciones: Receiver<Instruccion>,
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
            // wait 100 microseconds
            thread::sleep(std::time::Duration::from_micros(500));
        }
    }

    /// Este método se ejecuta en cada ciclo del hilo.
    /// Se encarga de procesar las instrucciones recibidas y
    /// realizar las acciones correspondientes.
    pub fn tick(&mut self) {
        self.recibir_conexiones();
        self.recibir_instrucciones();
        self.tick_conexiones();
        self.eliminar_conexiones_terminadas();
    }

    // Mientras se reciban conexiones,
    // con su id y la conexion correspondiente, el registrador informa
    // un evento de informacion con un registro con mensaje, hilo, y
    // nivel de tipo "Informacion". Ademas, se insertan en las
    // conexiones del hilo el id de la conexion y la conexion
    pub fn recibir_conexiones(&mut self) {
        while let Ok((id_conexion, conexion)) = self.canal_recibir_conexiones.try_recv() {
            self.registrador
                .info(&format!("Recibida conexión con id {}", id_conexion), None);
            self.conexiones.insert(id_conexion, conexion);
        }
    }

    // Mientras se reciban instrucciones, el registrador informa un evento de informacio
    // con un registro con mensaje, hilo y nivel de tipo "Información".
    pub fn recibir_instrucciones(&mut self) {
        while let Ok(instruccion) = self.canal_recibir_instrucciones.try_recv() {
            self.recibir_instruccion(instruccion);
        }
    }

    pub fn recibir_instruccion(&mut self, instruccion: Instruccion) {
        match instruccion {
            Instruccion::Suscribir(suscripcion) => {
                self.suscripciones.suscribir(suscripcion);
            }
            Instruccion::Desuscribir(id_conexion, id_suscripcion) => {
                self.suscripciones.desuscribir(id_conexion, &id_suscripcion);
            }
            Instruccion::Publicar(publicacion) => {
                self.recibir_publicacion(publicacion);
            }
            Instruccion::PublicarExacto(suscripcion, publicacion) => {
                self.recibir_publicacion_exacto(&suscripcion, publicacion);
            }
            _ => {}
        }
    }

    pub fn recibir_publicacion(&mut self, publicacion: Publicacion) {
        // Iterar sobre las suscripciones y enviar la publicación a cada una
        // Cabe destacar que solo itera en las suscripciones que coinciden con el tópico de la publicación
        for suscripcion in self.suscripciones.suscripciones_topico(&publicacion.topico) {
            if let Some(conexion) = self.conexiones.get_mut(suscripcion.id_conexion()) {
                conexion.escribir_publicacion_mensaje(
                    &publicacion.mensaje(suscripcion.id().to_owned()),
                );
            } else {
                self.registrador.error(
                    "No se encontró una conexión que debería existir",
                    Some(*suscripcion.id_conexion()),
                )
            }
        }
    }

    pub fn recibir_publicacion_exacto(
        &mut self,
        suscripcion: &Suscripcion,
        publicacion: Publicacion,
    ) {
        if let Some(conexion) = self.conexiones.get_mut(suscripcion.id_conexion()) {
            conexion
                .escribir_publicacion_mensaje(&publicacion.mensaje(suscripcion.id().to_owned()));
        }
    }

    pub fn tick_conexiones(&mut self) {
        let mut salidas = Vec::new();

        for (id_conexion, conexion) in self.conexiones.iter_mut() {
            let mut tick_salida = TickContexto::new(self.id, *id_conexion);
            conexion.tick(&mut tick_salida);
            salidas.push(tick_salida);
        }

        for salida in salidas {
            for instruccion in salida.instrucciones {
                match instruccion {
                    Instruccion::NuevaPublicacion(publicacion) => {
                        self.registrador.info(
                            &format!("Nueva publicacion: {:?}", &publicacion),
                            Some(salida.id_conexion),
                        );

                        self.enviar_instruccion_publicar(publicacion.clone());
                    }
                    Instruccion::Suscribir(suscripcion) => {
                        self.registrador.info(
                            &format!("Nueva suscripción: {:?}", &suscripcion),
                            Some(salida.id_conexion),
                        );

                        self.enviar_instruccion(Instruccion::Suscribir(suscripcion.clone()));
                        self.suscripciones.suscribir(suscripcion);
                    }
                    Instruccion::Desuscribir(id_conexion, id_suscripcion) => {
                        self.registrador.info(
                            &format!(
                                "Nueva desuscripcion: id_con={} id_sub={}",
                                &id_conexion, &id_suscripcion
                            ),
                            Some(salida.id_conexion),
                        );

                        self.suscripciones.desuscribir(id_conexion, &id_suscripcion);
                        self.enviar_instruccion(Instruccion::Desuscribir(
                            id_conexion,
                            id_suscripcion,
                        ));
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn enviar_instruccion(&self, instruccion: Instruccion) {
        for (id_hilo, tx) in self.canales_enviar_instrucciones.iter() {
            if id_hilo.eq(&self.id) {
                continue;
            }
            let r = tx.send(instruccion.clone());
            if r.is_err() {
                self.registrador
                    .error("No se pudo enviar la instrucción a otro proceso", None);
            }
        }
    }

    pub fn enviar_instruccion_publicar(&mut self, publicacion: Publicacion) {
        let hilos = self
            .suscripciones
            .hilos_suscriptos_topico(&publicacion.topico);

        for hilo in hilos {
            if hilo.eq(&self.id) {
                self.recibir_publicacion(publicacion.clone());
                continue;
            }

            if let Some(tx) = self.canales_enviar_instrucciones.get(&hilo) {
                let r = tx.send(Instruccion::Publicar(publicacion.clone()));
                if r.is_err() {
                    self.registrador
                        .error("No se pudo enviar la instrucción a otro proceso", None);
                }
            }
        }

        for grupo in self.suscripciones.grupos_topico(&publicacion.topico) {
            if let Some(suscripcion) = grupo.suscripcion_random() {
                if let Some(tx) = self.canales_enviar_instrucciones.get(suscripcion.id_hilo()) {
                    let r = tx.send(Instruccion::PublicarExacto(
                        suscripcion.clone(),
                        publicacion.clone(),
                    ));
                    if r.is_err() {
                        self.registrador
                            .error("No se pudo enviar la instrucción a otro proceso", None);
                    }
                }
            }
        }
    }

    pub fn eliminar_conexiones_terminadas(&mut self) {
        let mut suscripciones_eliminar = Vec::new();

        self.conexiones.retain(|id_conexion, conexion| {
            let esta_conextado = conexion.esta_conectado();

            if !esta_conextado {
                self.registrador
                    .info("Conexión terminada", Some(*id_conexion));

                for suscripcion in self.suscripciones.suscripciones_conexion(id_conexion) {
                    suscripciones_eliminar.push((*id_conexion, suscripcion.id().to_owned()));
                }
            }

            esta_conextado
        });

        for (id_conexion, id_suscripcion) in suscripciones_eliminar {
            self.suscripciones.desuscribir(id_conexion, &id_suscripcion);
            self.enviar_instruccion(Instruccion::Desuscribir(id_conexion, id_suscripcion));
        }
    }
}

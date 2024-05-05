use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

use rand::Rng;

use crate::{
    conexion::Conexion,
    instrucciones::Instrucciones,
    publicacion::{self, Publicacion},
    topico::Topico,
};

pub struct Proceso {
    /// Id del proceso
    id: u64,
    /// Canales a otros procesos para **enviar** instrucciones (ejemplo: publicar, suscribir, desuscribir, etc.)
    otros_procesos: HashMap<u64, Sender<Instrucciones>>,
    /// Canal para **recibir** instrucciones de otros procesos
    canal_instrucciones: Receiver<Instrucciones>,
    /// Canal para recibir nuevas conexiones
    canal_nuevas_conexiones: Receiver<Conexion>,
    /// Conexiones indexadas por id (el id es un número incremental generado cuando se crea la conexión)
    conexiones: HashMap<u64, Conexion>,
    /// Suscripciones locales, por tópico, por cada cliente
    suscripciones_locales: HashMap<Topico, HashSet<u64>>,
    /// Suscripciones de cola (queue groups). (nombre, (tópico, id))
    suscripciones_colas_locales: HashMap<String, (Topico, HashSet<u64>)>,
    /// Suscripciones a otros procesos, por tópico, por cada thread
    suscripciones_otros: HashMap<Topico, HashSet<u64>>,
    /// Suscripciones de cola (queue groups) a otros procesos. (nombre, (tópico, (id proceso, peso)))
    suscripciones_colas_otros: HashMap<String, (Topico, HashMap<u64, u64>)>,
}

impl Proceso {
    pub fn new(
        id: u64,
        otros_procesos: HashMap<u64, Sender<Instrucciones>>,
        canal_instrucciones: Receiver<Instrucciones>,
        canal_nuevas_conexiones: Receiver<Conexion>,
    ) -> Proceso {
        Proceso {
            id,
            otros_procesos,
            canal_instrucciones,
            canal_nuevas_conexiones,
            conexiones: HashMap::new(),
            suscripciones_locales: HashMap::new(),
            suscripciones_colas_locales: HashMap::new(),
            suscripciones_otros: HashMap::new(),
            suscripciones_colas_otros: HashMap::new(),
        }
    }

    pub fn iniciar(mut proceso: Proceso) -> JoinHandle<()> {
        thread::spawn(move || {
            // Crea un nuevo hilo y ejecuta proceso.inicio() dentro
            proceso.inicio();
        })
    }

    pub fn inicio(&mut self) {
        loop {
            self.tick();
        }
    }

    pub fn enviar_instruccion(&self, instruccion: Instrucciones) {
        for (id, tx) in &self.otros_procesos {
            let r = tx.send(instruccion.clone());
            if r.is_err() {
                panic!("No se pudo enviar la instrucción a otro proceso");
            }
        }
    }

    pub fn procesos_suscritos_a(&self, topico: &str) -> Vec<u64> {
        let mut procesos = Vec::new();
        for (t, ids) in &self.suscripciones_otros {
            if t.test(topico) {
                procesos.extend(ids.iter());
            }
        }

        procesos
    }

    pub fn colas_subscritas_a(&self, topico: &str) -> Vec<(String, u64, u64)> {
        let mut colas = Vec::new();
        for (cola, (t, ids)) in &self.suscripciones_colas_otros {
            if t.test(topico) {
                for (id, peso) in ids {
                    colas.push((cola.clone(), *id, *peso));
                }
            }
        }

        colas
    }

    /// Envia las publicaciones a los otros procesos que correspondan
    pub fn publicar(&mut self, publicacion: &Publicacion) {
        self.publicar_local(&publicacion);
    }

    /// Publica los mensajes a las conexiones de este proceso
    pub fn publicar_local(&mut self, publicacion: &Publicacion) {
        for (topico, conexiones) in &self.suscripciones_locales {
            if topico.test(&publicacion.topico) {
                for id in conexiones {
                    if let Some(conexion) = self.conexiones.get_mut(&id) {
                        conexion.recibir_mensaje(publicacion.clone());
                    }
                }
            }
        }
    }

    /// Publica el mensaje a un suscriptor de la cola
    pub fn publicar_local_cola(&mut self, cola: &str, publicacion: &Publicacion) {
        if let Some((topico, conexiones)) = self.suscripciones_colas_locales.get(cola) {
            if topico.test(&publicacion.topico) {
                // elegir conexion random
                let values = conexiones.iter().collect::<Vec<&u64>>();
                rand::thread_rng().gen_range(0..conexiones.len());

                if let Some(conexion) = self.conexiones.get_mut(values[0]) {
                    conexion.recibir_mensaje(publicacion.clone());
                }
            }
        }
    }

    pub fn publicar_colas(&mut self, publicacion: &Publicacion) {
        let mut nuevas_para_colas_locales = Vec::new();

        for (cola, (topico, otros_pesos)) in &mut self.suscripciones_colas_otros {
            if topico.test(&publicacion.topico) {
                if let Some((id, peso)) = elegir_si_proceso_local_u_otro_random(otros_pesos, 0) {
                    if let Some(tx) = self.otros_procesos.get(&id) {
                        let r = tx.send(Instrucciones::PublicarCola(
                            cola.clone(),
                            publicacion.clone(),
                        ));
                        if r.is_err() {
                            panic!("No se pudo enviar la instrucción a otro proceso");
                        }
                    }
                } else {
                    nuevas_para_colas_locales.push((cola.clone(), publicacion.clone()));
                }
            }
        }

        for (cola, publicacion) in &nuevas_para_colas_locales {
            self.publicar_local_cola(cola, publicacion);
        }
    }

    pub fn tick(&mut self) {
        // Itear instrucciones
        while let Ok(instruccion) = self.canal_instrucciones.try_recv() {
            self.gestionar_instruccion(instruccion);
        }

        let mut nuevas_publicaciones = Vec::new();

        // Iterar conexiones
        for (id, conexion) in &mut self.conexiones {
            conexion.tick();

            // Extraer las publicaciones salientes que se pueden haber generado en el tick
            let publicaciones_salientes = conexion.extraer_publicaciones_salientes();

            nuevas_publicaciones.extend(publicaciones_salientes);
        }

        for publicacion in &nuevas_publicaciones {
            self.publicar(publicacion);
            self.publicar_colas(publicacion);
        }

        self.conexiones.retain(|id, c| {
            if c.desconectado {
                self.suscripciones_locales.retain(|_, ids| {
                    ids.remove(id);
                    !ids.is_empty()
                });

                self.suscripciones_colas_locales.retain(|_, (_, ids)| {
                    ids.remove(id);
                    !ids.is_empty()
                });

                false
            } else {
                true
            }
        });
    }

    pub fn gestionar_instruccion(&mut self, instruccion: Instrucciones) {
        match instruccion {
            Instrucciones::Suscribir(id, topico) => {
                self.suscripciones_otros
                    .entry(topico)
                    .or_insert(HashSet::new())
                    .insert(id);
            }
            Instrucciones::Desuscribir(id) => {
                self.suscripciones_otros.retain(|_, ids| {
                    ids.remove(&id);
                    !ids.is_empty()
                });
            }
            Instrucciones::ActualizarCola {
                id,
                cola,
                topico,
                peso,
            } => {
                if peso == 0 {
                    if let Some((_, otros)) = &mut self.suscripciones_colas_otros.get_mut(&cola) {
                        otros.remove(&id);
                        if otros.is_empty() {
                            self.suscripciones_colas_otros.remove(&cola);
                        }
                    }
                } else {
                    self.suscripciones_colas_otros
                        .entry(cola)
                        .or_insert((topico, HashMap::new()))
                        .1
                        .insert(id, peso);
                }
            }
            Instrucciones::Publicar( publicacion) => {
                self.publicar_local(&publicacion);
            }
            Instrucciones::PublicarCola(nombre, publicacion) => {
                self.publicar_local_cola(&nombre, &publicacion);
            }
        }
    }
}

// use std::{
//     sync::mpsc::{Receiver, Sender},
//     thread::{self, JoinHandle},
// };

// use super::{conexion::Conexion, publicacion::Publicacion};

// pub struct Proceso {
//     /// Canales a otros procesos
//     otros_procesos: Vec<Sender<Publicacion>>, // El tipo sender es un canal de solo escritura
//     nuevos_mensajes: Receiver<Publicacion>, // El tipo receiver es un canal de solo lectura
//     nuevas_conexiones: Receiver<Conexion>,
//     conexiones: Vec<Conexion>,
// }

// impl Proceso {
//     pub fn new(
//         otros_procesos: Vec<Sender<Publicacion>>,
//         canal: Receiver<Publicacion>,
//         nuevas_conexiones: Receiver<Conexion>,
//     ) -> Proceso {
//         Proceso {
//             otros_procesos,
//             nuevos_mensajes: canal,
//             nuevas_conexiones,
//             conexiones: Vec::new(),
//         }
//     }

//     pub fn iniciar(mut proceso: Proceso) -> JoinHandle<()> {
//         thread::spawn(move || { // Crea un nuevo hilo y ejecuta proceso.inicio() dentro
//             proceso.inicio();
//         })
//     }

//     fn inicio(&mut self) {
//         loop {
//             // Recibir nuevas conexiones iniciadas al servidor
//             while let Ok(conexion) = self.nuevas_conexiones.try_recv() {
//                 self.conexiones.push(conexion);
//             }

//             // Todas las publicaciones que se recibieron en esta iteración
//             // Generadas por las conexiones de este proceso
//             // Y por las conexiones de los otros procesos
//             let mut todas_las_publicaciones = Vec::new();

//             // Por cada conexión,
//             for conexion in &mut self.conexiones {
//                 // Ejecutar el tick (cada tick recibe, procesa y envía mensajes pendientes)
//                 conexion.tick();

//                 // Extraer las publicaciones salientes que se pueden haber generado en el tick
//                 let publicaciones_salientes = conexion.extraer_publicaciones_salientes();

//                 // Agregar las publicaciones salientes al vector de publicaciones
//                 todas_las_publicaciones.extend(publicaciones_salientes);
//             }

//             // Enviamos todas las publicaciones creadas por este proceso a los otros procesos
//             for tx in self.otros_procesos.iter() {
//                 for publicacion in &todas_las_publicaciones {
//                     let r = tx.send(publicacion.clone());
//                     if r.is_err() {
//                         panic!("No se pudo enviar la publicación a otro proceso");
//                     }
//                 }
//             }

//             // Recibir nuevas publicaciones de otros procesos
//             while let Ok(publicacion) = self.nuevos_mensajes.try_recv() {
//                 todas_las_publicaciones.push(publicacion);
//             }

//             // Por cada conexión y por cada cliente, enviarle todas las publicaciones
//             // Cada conexión decide si debe enviar el mensaje o ignorarlo según el tópico
//             for conexion in &mut self.conexiones {
//                 for publicacion in &todas_las_publicaciones {
//                     conexion.recibir_mensaje(publicacion.clone());
//                 }
//             }

//             self.conexiones.retain(|conexion| !conexion.desconectado);
//         }
//     }
// }

/// Dado un hashmap de otros proceso, con diferentes pesos, y el peso del thread local, elegir uno al azar
pub fn elegir_si_proceso_local_u_otro_random(
    otros_procesos: &HashMap<u64, u64>,
    peso_local: u64,
) -> Option<(u64, u64)> {
    let total = otros_procesos.values().sum::<u64>() + peso_local;
    let mut rng = rand::thread_rng();
    let mut acumulado = 0;

    let random = rng.gen_range(0..total);

    if random < peso_local {
        return None;
    }

    let random = random - peso_local;

    for (id, peso) in otros_procesos {
        acumulado += peso;
        if acumulado >= random {
            return Some((*id, *peso));
        }
    }

    None
}

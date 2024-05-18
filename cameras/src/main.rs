// use messaging::client::NATSClient;

use std::sync::mpsc::{Receiver, Sender};

use cameras::{
    camara::{id::IdCamara, Camara},
    estado::Estado,
    interfaz::{comando::Comando, interfaz, respuesta::Respuesta},
};
use messaging_client::cliente::Cliente;

fn main() {
    let mut estado = Estado::new();
    let (enviar_respuesta, recibir_comandos) = interfaz();

    loop {
        if let Err(e) = iniciar(&mut estado, &enviar_respuesta, &recibir_comandos) {
            eprintln!("Error en hilo principal: {}", e);
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}

fn iniciar(
    estado: &mut Estado,
    enviar_respuesta: &Sender<Respuesta>,
    recibir_comandos: &Receiver<Comando>,
) -> Result<(), String> {
    // 1. Conectar cliente
    let mut cliente = match Cliente::conectar("127.0.0.1:4222") {
        Ok(cliente) => cliente,
        Err(e) => return Err(e.to_string()),
    };

    // 2. Enviar estado de todas las camaras
    // 3. suscribir a cosas relevantes
    let mut suscripcion_incidentes = match cliente.suscribirse("incidentes.*.creado", None) {
        Ok(suscripcion) => suscripcion,
        Err(e) => return Err(e.to_string()),
    };

    loop {
        // Leer nats
        match suscripcion_incidentes.intentar_leer() {
            Ok(Some(publicacion)) => {}
            Err(e) => return Err(e.to_string()),
            _ => {}
        }

        // Leer comandos
        if let Ok(comando) = recibir_comandos.try_recv() {
            match comando {
                Comando::Conectar(id, lat, lon, rango) => {
                    let camara = Camara::new(id, lat, lon, rango);
                    match estado.conectar_camara(camara) {
                        Ok(()) => {
                            if let Err(e) = cliente.publicar(
                                &format!("camaras.{}.conectada", id),
                                &vec![],
                                None,
                            ) {
                                return Err(e.to_string());
                            }

                            enviar_respuesta.send(Respuesta::Ok).unwrap()
                        }
                        Err(e) => enviar_respuesta.send(Respuesta::Error(e)).unwrap(),
                    }
                }
                Comando::Desconectar(id) => match estado.desconectar_camara(id) {
                    Ok(()) => {
                        if let Err(e) =
                            cliente.publicar(&format!("camaras.{}.desconectada", id), &vec![], None)
                        {
                            return Err(e.to_string());
                        }

                        enviar_respuesta.send(Respuesta::Ok).unwrap()
                    }
                    Err(e) => enviar_respuesta.send(Respuesta::Error(e)).unwrap(),
                },
                Comando::ListarCamaras => {
                    let camaras: Vec<Camara> = estado.camaras().into_iter().cloned().collect();
                    if camaras.is_empty() {
                        enviar_respuesta
                            .send(Respuesta::Error("No hay cámaras conectadas".to_string()))
                            .unwrap();
                    } else {
                        enviar_respuesta.send(Respuesta::Camaras(camaras)).unwrap();
                    }
                }
                Comando::Camara(id) => {
                    if let Some(camara) = estado.camara(id) {
                        enviar_respuesta
                            .send(Respuesta::Camara(camara.clone()))
                            .unwrap();
                    } else {
                        enviar_respuesta
                            .send(Respuesta::Error(
                                "No existe una cámara con ese ID".to_string(),
                            ))
                            .unwrap();
                    }
                }
                Comando::ModificarUbicacion(id, lat, lon) => {
                    match estado.modificar_ubicacion(id, lat, lon) {
                        Ok(()) => {
                            if let Err(e) = cliente.publicar(
                                &format!("camaras.{}.conectada", id),
                                &vec![],
                                None,
                            ) {
                                return Err(e.to_string());
                            }
                            enviar_respuesta.send(Respuesta::Ok).unwrap()
                        }
                        Err(e) => enviar_respuesta.send(Respuesta::Error(e)).unwrap(),
                    }
                }
                Comando::ModifciarRango(id, rango) => match estado.modificar_rango(id, rango) {
                    Ok(()) => {
                        if let Err(e) =
                            cliente.publicar(&format!("camaras.{}.conectada", id), &vec![], None)
                        {
                            return Err(e.to_string());
                        }

                        enviar_respuesta.send(Respuesta::Ok).unwrap();
                    }
                    Err(e) => enviar_respuesta.send(Respuesta::Error(e)).unwrap(),
                },
                Comando::Ayuda => {
                    enviar_respuesta.send(Respuesta::Ayuda).unwrap();
                }
            }
        }
    }
}

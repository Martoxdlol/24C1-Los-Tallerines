// use messaging::client::NATSClient;

use std::ops::RangeBounds;

use cameras::{
    camara::{id, Camara},
    estado::Estado,
    interfaz::{comando::Comando, interfaz, respuesta::Respuesta},
};

fn main() {
    let mut estado = Estado::new();
    let (enviar_respuesta, recibir_comandos) = interfaz();

    loop {
        // Leer nats

        // Leer comandos
        if let Ok(comando) = recibir_comandos.try_recv() {
            match comando {
                Comando::Conectar(id, lat, lon, rango) => {
                    let camara = Camara::new(id, lat, lon, rango);
                    match estado.conectar_camara(camara) {
                        Ok(()) => enviar_respuesta.send(Respuesta::Ok).unwrap(),
                        Err(e) => enviar_respuesta.send(Respuesta::Error(e)).unwrap(),
                    }
                }
                Comando::Desconectar(id) => match estado.desconectar_camara(id) {
                    Ok(()) => enviar_respuesta.send(Respuesta::Ok).unwrap(),
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
                Comando::MostrarCamara(id) => {
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
                        Ok(()) => enviar_respuesta.send(Respuesta::Ok).unwrap(),
                        Err(e) => enviar_respuesta.send(Respuesta::Error(e)).unwrap(),
                    }
                }
                Comando::ModifciarRango(id, rango) => match estado.modificar_rango(id, rango) {
                    Ok(()) => enviar_respuesta.send(Respuesta::Ok).unwrap(),
                    Err(e) => enviar_respuesta.send(Respuesta::Error(e)).unwrap(),
                },
                Comando::Ayuda => {
                    enviar_respuesta.send(Respuesta::Ayuda).unwrap();
                }
            }
        }
    }
}

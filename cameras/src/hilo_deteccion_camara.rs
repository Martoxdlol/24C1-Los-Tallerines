use rand::seq::SliceRandom;
use rekognition::reconocer_imagen;
use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
}; // 0.7.2

use lib::{camara::Camara, deteccion::Deteccion};

pub struct HiloDeteccionCamara {
    camara: Camara,
    ruta: String,
    enviar_deteccion: Sender<Deteccion>,
    detener_deteccion: Receiver<()>,
}

impl HiloDeteccionCamara {
    pub fn new(
        camara: Camara,
        ruta: String,
        enviar_deteccion: Sender<Deteccion>,
        detener_deteccion: Receiver<()>,
    ) -> Self {
        Self {
            camara,
            ruta,
            enviar_deteccion,
            detener_deteccion,
        }
    }

    pub fn iniciar(&self) {
        loop {
            if let Ok(()) = self.detener_deteccion.try_recv() {
                break;
            }

            if let Err(e) = self.ciclo() {
                eprintln!(
                    "Error en el ciclo de detección de la cámara {}: {}",
                    self.camara.id, e
                );
            }

            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    pub fn ciclo(&self) -> Result<(), String> {
        let archivos = std::fs::read_dir(&self.ruta).map_err(|e| e.to_string())?;

        let imagenes = archivos
            .filter_map(|archivo| {
                if let Ok(archivo) = archivo {
                    if es_imagen(&archivo) {
                        Some(archivo)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if imagenes.len() == 0 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            return Ok(());
        }

        let imagen_random = imagenes.choose(&mut rand::thread_rng()).unwrap();

        let resultado = match reconocer_imagen(imagen_random.path()) {
            Ok(resultado) => resultado,
            Err(e) => {
                std::fs::remove_file(imagen_random.path()).map_err(|e| e.to_string())?;
                return Err(e);
            }
        };

        std::fs::remove_file(imagen_random.path()).map_err(|e| e.to_string())?;

        let mut etiquetas = HashMap::new();

        for label in resultado {
            if let Some(name) = label.name {
                etiquetas.insert(name, label.confidence.unwrap_or(1.) as f64);
            }
        }

        let deteccion = Deteccion {
            id_camara: self.camara.id,
            etiquetas,
            posicion: self.camara.posicion_en_rango_aleatoria(),
        };

        self.enviar_deteccion
            .send(deteccion)
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

pub fn es_imagen(archivo: &std::fs::DirEntry) -> bool {
    if let Some(extension) = archivo.path().extension() {
        if let Some(extension) = extension.to_str() {
            return extension == "jpg"
                || extension == "jpeg"
                || extension == "png"
                || extension == "webp"
                || extension == "gif"
                || extension == "avif";
        }
    }

    false
}

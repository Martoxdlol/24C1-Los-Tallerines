use crate::{csv::{csv_encodear_linea, csv_parsear_linea}, serializables::{error::DeserializationError, Serializable}};

use std::{sync::mpsc::Sender, thread, time::Duration};

#[derive(Debug, Clone)]
pub struct Bateria {
    pub bateria_inicial: u64,
    pub bateria_actual: u64,
    pub duracion_bateria: u64, // En segundos
    pub bateria_minima: u64,
    pub canal_notificar_bateria: Option<Sender<u64>>,
}

impl Bateria {
    pub fn new(bateria_inicial: u64, bateria_actual: u64, duracion_bateria: u64, bateria_minima: u64, canal_notificar_bateria: Sender<u64>) -> Self {
        Bateria {
            bateria_inicial,
            bateria_actual,
            duracion_bateria,
            bateria_minima,
            canal_notificar_bateria: Some(canal_notificar_bateria),
        }
    }

    pub fn necesita_recargarse(&mut self) -> bool {
        self.bateria_actual <= self.bateria_minima
    }

    pub fn nivel_actual(&mut self) -> u64 {
        self.bateria_actual
    }

    /*
    pub fn descargar(&mut self) {
        println!(
            "Bateria minima: {}, duracion de la bateria: {}",
            self.bateria_minima, self.duracion_bateria
        );

        thread::spawn(move || {
            // Ejemplo:
            // bateria inicial = 100
            // bateria_minima = 30, 
            // duracion_bateria = 35, 
            // descarga_por_segundo = (100 - 30) / 35 = 2
            let descarga_por_segundo: u64 = (self.bateria_inicial - self.bateria_minima) / self.duracion_bateria;

            while !self.necesita_recargarse() {
                thread::sleep(Duration::from_secs(descarga_por_segundo));

                self.bateria_actual -= descarga_por_segundo;

                println!("Nivel de bateria: {}%", self.bateria_actual);

                if self.necesita_recargarse() {
                    self.canal_notificar_bateria.send(self.bateria_actual).unwrap();
                }
            }
        });
    }
    */
}

impl Serializable for Bateria {
    fn serializar(&self) -> Vec<u8> {
        let mut parametros: Vec<String> = Vec::new();
        parametros.push(format!("{}", self.duracion_bateria));
        parametros.push(format!("{}", self.bateria_minima));
        csv_encodear_linea(&parametros).into_bytes()
    }

    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError> {
        let linea =
            String::from_utf8(data.to_vec()).map_err(|_| DeserializationError::InvalidData)?;
        let mut parametros = csv_parsear_linea(linea.as_str()).into_iter();

        let bateria_inicial = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let bateria_actual = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let duracion_bateria = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let bateria_minima = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        Ok(Bateria {
            bateria_inicial,
            bateria_actual,
            duracion_bateria, 
            bateria_minima,
            canal_notificar_bateria: None,
        })
    }
}
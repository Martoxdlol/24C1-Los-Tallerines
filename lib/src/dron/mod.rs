use std::str::FromStr;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use std::{collections::HashSet, io};

use crate::{
    configuracion::Configuracion,
    csv::{csv_encodear_linea, csv_parsear_linea},
    serializables::{error::DeserializationError, guardar::cargar_serializable, Serializable},
};

#[derive(Debug, Clone)]
pub enum EstadoDron {
    EnEspera,
    VolviendoACentroDeOperacion,
    YendoAIncidente,
    AtendiendoIncidente,
    YendoACentral,
    CargandoEnCentral,
    Error,
}

#[derive(Debug, Clone)]
pub struct Dron {
    pub id: u64,
    pub latitud: f64,
    pub longitud: f64,
    pub rango: f64,
    estado: EstadoDron,
    direccion: f64, // En grados, sentido horario, empezando desde el norte
    velocidad: f64,
    pub duracion_bateria: u64, // En segundos
    pub bateria_minima: u64,
    pub incidentes_cercanos: HashSet<u64>,
    latitud_central: f64,
    longitud_central: f64,
    latitud_centro_operaciones: f64,
    longitud_centro_operaciones: f64,
    pub configuracion: Configuracion,
}

impl Dron {
    /// Se inicia el dron sin valores concretos, solo se inicia la configuración
    pub fn new(configuracion: Configuracion) -> Self {
        Dron {
            id: 0,
            latitud: 0.0,
            longitud: 0.0,
            rango: 0.0,
            estado: EstadoDron::EnEspera,
            direccion: 0.0,
            velocidad: 0.0,
            duracion_bateria: 0,
            bateria_minima: 0,
            incidentes_cercanos: HashSet::new(),
            latitud_central: 0.0,
            longitud_central: 0.0,
            latitud_centro_operaciones: 0.0,
            longitud_centro_operaciones: 0.0,
            configuracion,
        }
    }

    /// Se carga el dron, se va descargando la bateria, y mientras el dron realiza sus acciones
    pub fn iniciar(&mut self) -> io::Result<()> {
        // Canal por el cual se va comunicando la bateria del dron
        let (tx_descarga_bateria, rx_descarga_bateria) = mpsc::channel::<u64>();
        self.cargar_dron(tx_descarga_bateria)?;


        let mut bateria_descargada = false;
        loop {
            bateria_descargada = self.verificar_nivel_de_bateria(&rx_descarga_bateria);
            if bateria_descargada {
                break;
            }
        }

        Ok(())
    }

    fn verificar_nivel_de_bateria(&mut self, rx_descarga_bateria: &Receiver<u64>) -> bool {
        match rx_descarga_bateria.try_recv() {
            Ok(bateria) => {
                println!("Alerta de nivel de bateria: {}%", bateria);
                return true
            }
            Err(mpsc::TryRecvError::Empty) => {
                // No hay mensaje en el canal
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                // El hilo de descarga ha terminado
                return true;
            }
        }

        thread::sleep(Duration::from_secs(2));

        false
    }

    /// Se carga información del dron desde una configuración, se carga esa
    /// información en el dron y se descarga la bateria del dron
    fn cargar_dron(&mut self, tx_descarga_bateria: Sender<u64>) -> io::Result<()> {
        let ruta_archivo_dron = self
            .configuracion
            .obtener::<String>("drones")
            .unwrap_or("dron.csv".to_string());

        let existe = std::path::Path::new(&ruta_archivo_dron).exists();

        if !existe {
            std::fs::File::create(&ruta_archivo_dron)?;
        }

        let dron: Dron = cargar_serializable(&ruta_archivo_dron)?;

        self.setear_valores(dron);
        println!("\nDron: {:?}", self);

        self.incidentes_cercanos.clear();
        self.descargar_bateria(tx_descarga_bateria);

        Ok(())
    }

    fn setear_valores(&mut self, dron: Dron) {
        self.id = dron.id;
        self.latitud = dron.latitud;
        self.longitud = dron.longitud;
        self.rango = dron.rango;
        self.estado = dron.estado;
        self.direccion = dron.direccion;
        self.velocidad = dron.velocidad;
        self.duracion_bateria = dron.duracion_bateria;
        self.bateria_minima = dron.bateria_minima;
        self.incidentes_cercanos = dron.incidentes_cercanos;
        self.latitud_central = dron.latitud_central;
        self.longitud_central = dron.longitud_central;
        self.latitud_centro_operaciones = dron.latitud_centro_operaciones;
        self.longitud_centro_operaciones = dron.longitud_centro_operaciones;
    }

    /// Se descarga la bateria hasta alcanzar el nivel minimo de bateria del dron
    fn descargar_bateria(&mut self, tx_descarga_bateria: Sender<u64>) {
        let bateria_minima = self.bateria_minima;
        let duracion_bateria = self.duracion_bateria;
        println!(
            "Bateria minima: {}, duracion de la bateria: {}",
            bateria_minima, duracion_bateria
        );

        thread::spawn(move || {
            // Ejemplo: bateria_minima = 30, duracion_bateria = 35, descarga_por_segundo = 2
            let descarga_por_segundo: u64 = (100 - bateria_minima) / duracion_bateria;
            let mut bateria: u64 = 100;

            while bateria > bateria_minima {
                thread::sleep(Duration::from_secs(descarga_por_segundo));

                bateria -= descarga_por_segundo;

                println!("Nivel de bateria: {}%", bateria);

                // Envío al hilo principal la bateria del dron
                if bateria <= bateria_minima {
                    tx_descarga_bateria.send(bateria).unwrap();
                }
            }
        });
    }
}

impl Serializable for Dron {
    fn serializar(&self) -> Vec<u8> {
        let mut parametros: Vec<String> = Vec::new();
        parametros.push(format!("{}", self.id));
        parametros.push(format!("{}", self.latitud));
        parametros.push(format!("{}", self.longitud));
        parametros.push(format!("{}", self.rango));
        parametros.push(format!("{:?}", self.estado));
        parametros.push(format!("{}", self.direccion));
        parametros.push(format!("{}", self.velocidad));
        parametros.push(format!("{}", self.duracion_bateria));
        parametros.push(serializar_vector_incidentes(&self.incidentes_cercanos).to_string());
        parametros.push(format!("{}", self.latitud_central));
        parametros.push(format!("{}", self.longitud_central));
        parametros.push(format!("{}", self.latitud_centro_operaciones));
        parametros.push(format!("{}", self.longitud_centro_operaciones));

        csv_encodear_linea(&parametros).into_bytes()
    }

    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError> {
        let linea =
            String::from_utf8(data.to_vec()).map_err(|_| DeserializationError::InvalidData)?;

        let mut parametros = csv_parsear_linea(linea.as_str()).into_iter();

        let id = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let latitud = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let longitud = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let rango = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let estado = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let direccion = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let velocidad = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let duracion_bateria = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let bateria_minima = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let incidentes_cercanos = deserialize_vector_incidentes(
            &parametros
                .next()
                .ok_or(DeserializationError::MissingField)?,
        )?;

        let latitud_central = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let longitud_central = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let latitud_centro_operaciones = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let longitud_centro_operaciones = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        Ok(Dron {
            id,
            latitud,
            longitud,
            rango,
            estado,
            direccion,
            velocidad,
            duracion_bateria,
            bateria_minima,
            incidentes_cercanos,
            latitud_central,
            longitud_central,
            latitud_centro_operaciones,
            longitud_centro_operaciones,
            configuracion: Configuracion::new(),
        })
    }
}

fn serializar_vector_incidentes(incidentes: &HashSet<u64>) -> String {
    incidentes
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(";")
}

fn deserialize_vector_incidentes(data: &str) -> Result<HashSet<u64>, DeserializationError> {
    if data.trim().is_empty() {
        return Ok(HashSet::new());
    }

    data.split(';')
        .map(|id| id.parse().map_err(|_| DeserializationError::InvalidData))
        .collect()
}

impl FromStr for EstadoDron {
    type Err = DeserializationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EnEspera" => Ok(EstadoDron::EnEspera),
            "VolviendoACentroDeOperacion" => Ok(EstadoDron::VolviendoACentroDeOperacion),
            "YendoAIncidente" => Ok(EstadoDron::YendoAIncidente),
            "AtendiendoIncidente" => Ok(EstadoDron::AtendiendoIncidente),
            "YendoACentral" => Ok(EstadoDron::YendoACentral),
            "CargandoEnCentral" => Ok(EstadoDron::CargandoEnCentral),
            _ => Err(DeserializationError::InvalidData),
        }
    }
}

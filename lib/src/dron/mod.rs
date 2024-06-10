pub mod accion;

use accion::Accion;

use crate::{
    configuracion::Configuracion,
    coordenadas::Coordenadas,
    incidente::Incidente,
    serializables::{deserializador::Deserializador, serializador::Serializador, Serializable},
};

#[derive(Clone, Debug)]
pub struct Dron {
    pub id: u64,
    pub rango: f64,
    pub central_de_carga: Coordenadas,
    pub punto_de_espera: Coordenadas,
    pub velocidad_maxima: f64,
    pub velocidad_descarga_bateria: f64,
    pub posicion: Coordenadas,
    pub direccion_actual: f64,
    pub bateria_actual: f64,
    pub velocidad_actual: f64,
    pub incidente_actual: Option<Incidente>,
    pub envio_ultimo_estado: i64,
}

impl Dron {
    pub fn crear(config: &Configuracion) -> Option<Self> {
        let central_de_carga_lat = config.obtener("central_de_carga.lat")?;
        let central_de_carga_lon = config.obtener("central_de_carga.lon")?;

        let central_de_carga =
            Coordenadas::from_lat_lon(central_de_carga_lat, central_de_carga_lon);

        let punto_de_espera_lat = config.obtener("punto_de_espera.lat")?;
        let punto_de_espera_lon = config.obtener("punto_de_espera.lon")?;

        let punto_de_espera = Coordenadas::from_lat_lon(punto_de_espera_lat, punto_de_espera_lon);

        Some(Dron {
            id: config.obtener("id")?,
            rango: config.obtener("rango").unwrap_or(1500.),
            bateria_actual: config.obtener("bateria").unwrap_or(100.),
            central_de_carga,
            direccion_actual: config.obtener("direccion").unwrap_or(0.),
            incidente_actual: None,
            posicion: Coordenadas::from_lat_lon(
                config.obtener("lat").unwrap_or(punto_de_espera_lat),
                config.obtener("lon").unwrap_or(punto_de_espera_lon),
            ),
            punto_de_espera,
            velocidad_maxima: config.obtener("velocidad_maxima").unwrap_or(10.),
            velocidad_actual: config.obtener("velocidad_actual").unwrap_or(0.),
            velocidad_descarga_bateria: config
                .obtener("velocidad_descarga_bateria")
                .unwrap_or(1. / 3600.),
            envio_ultimo_estado: 0,
        })
    }
}

impl Dron {
    pub fn accion(&self) -> Accion {
        if self.bateria_actual < 10. {
            return Accion::Cargar;
        }

        if let Some(incidente) = &self.incidente_actual {
            return Accion::Incidente(incidente.clone());
        }

        Accion::Espera
    }

    pub fn destino(&self) -> Coordenadas {
        match self.accion() {
            Accion::Incidente(incidente) => incidente.posicion(),
            Accion::Cargar => self.central_de_carga,
            Accion::Espera => self.punto_de_espera,
        }
    }

    pub fn predecir_posicion(&self, tiempo: f64) -> Coordenadas {
        let distancia = self.velocidad_actual * tiempo;
        let destino = self.destino();
        let distancia_destino = self.posicion.distancia(&destino);

        if distancia_destino <= distancia {
            return destino;
        }

        let direccion = self.posicion.direccion(&destino);
        

        self.posicion.mover_en_direccion(distancia, direccion)
    }
}

impl Serializable for Dron {
    fn deserializar(data: &[u8]) -> Result<Self, crate::serializables::error::DeserializationError>
    where
        Self: Sized,
    {
        let mut deserializador = Deserializador::new(data.to_vec());

        let id = deserializador.sacar_elemento()?;
        let rango = deserializador.sacar_elemento()?;
        let central_de_carga = deserializador.sacar_elemento_serializable()?;
        let punto_de_espera = deserializador.sacar_elemento_serializable()?;
        let velocidad_maxima = deserializador.sacar_elemento()?;
        let velocidad_descarga_bateria = deserializador.sacar_elemento()?;
        let posicion = deserializador.sacar_elemento_serializable()?;
        let direccion_actual = deserializador.sacar_elemento()?;
        let bateria_actual = deserializador.sacar_elemento()?;
        let velocidad_actual = deserializador.sacar_elemento()?;
        let incidente_actual = deserializador.sacar_elemento_serializable()?;
        let envio_ultimo_estado = deserializador.sacar_elemento()?;

        Ok(Dron {
            id,
            rango,
            central_de_carga,
            punto_de_espera,
            velocidad_maxima,
            velocidad_descarga_bateria,
            posicion,
            direccion_actual,
            bateria_actual,
            velocidad_actual,
            incidente_actual,
            envio_ultimo_estado,
        })
    }

    fn serializar(&self) -> Vec<u8> {
        let mut serializador = Serializador::new();

        serializador.agregar_elemento(&self.id);
        serializador.agregar_elemento(&self.rango);
        serializador.agregar_elemento_serializable(&self.central_de_carga);
        serializador.agregar_elemento_serializable(&self.punto_de_espera);
        serializador.agregar_elemento(&self.velocidad_maxima);
        serializador.agregar_elemento(&self.velocidad_descarga_bateria);
        serializador.agregar_elemento_serializable(&self.posicion);
        serializador.agregar_elemento(&self.direccion_actual);
        serializador.agregar_elemento(&self.bateria_actual);
        serializador.agregar_elemento(&self.velocidad_actual);
        serializador.agregar_elemento_serializable(&self.incidente_actual);
        serializador.agregar_elemento(&self.envio_ultimo_estado);

        serializador.bytes
    }
}

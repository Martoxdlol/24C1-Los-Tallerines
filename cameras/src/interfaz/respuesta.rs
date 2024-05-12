use crate::camara::Camara;

pub enum Respuesta {
    Ok,
    Error(String),
    Camaras(Vec<Camara>),
    Camara(Camara),
    Ayuda,
}

impl Respuesta {
    pub fn ok() -> Self {
        Respuesta::Ok
    }

    pub fn error<T: Into<String>>(error: T) -> Self {
        Respuesta::Error(error.into())
    }

    pub fn como_string(&self) -> String {
        match self {
            Respuesta::Ok => "Ok".to_string(),
            Respuesta::Error(error) => format!("Error: {}", error),
            Respuesta::Camaras(camaras) => self.camaras_string(camaras),
            Respuesta::Ayuda => "conectar <ID> <Lat> <Lon> <Rango>\ndesconectar <ID>\nlistar\nmodificar ubicacion <ID> <Lat> <Lon>\nmodificar rango <ID> <Rango>\nayuda".to_string(),
            Respuesta::Camara(camara) => self.camara_string(camara),
        }
    }

    fn camara_string(&self, camara: &Camara) -> String {
        format!(
            "ID: {}, Lat: {}, Lon: {}, Estado: {}",
            camara.id,
            camara.lat,
            camara.lon,
            camara.estado()
        )
    }

    fn camaras_string(&self, camaras: &[Camara]) -> String {
        let lineas = camaras
            .iter()
            .map(|c| self.camara_string(c))
            .collect::<Vec<String>>();
        lineas.join("\n")
    }
}

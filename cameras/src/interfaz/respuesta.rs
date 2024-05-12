use crate::camara::Camara;

pub enum Respuesta {
    Ok,
    Error(String),
    Camaras(Vec<Camara>),
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
        }
    }

   fn camaras_string(&self, camaras: &Vec<Camara>) -> String {
        let mut estados = String::new();
        for camara in camaras {
            estados.push_str(&format!("ID: {}, Lat: {}, Lon: {}, Estado: {}\n", camara.id, camara.lat, camara.lon, camara.estado()));
        }
        estados.trim_end().to_string()
    }
}
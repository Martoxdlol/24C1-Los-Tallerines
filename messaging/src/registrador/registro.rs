#[derive(Debug, Clone)]
pub struct Registro {
    pub nivel: NivelRegistro,
    pub hilo: Option<u64>,
    pub conexion: Option<u64>,
    pub mensaje: String,
}

impl Registro {
    pub fn info(mensaje: String, hilo: Option<u64>, conexion: Option<u64>) -> Registro {
        Registro {
            nivel: NivelRegistro::Informacion,
            hilo,
            conexion,
            mensaje,
        }
    }

    pub fn advertencia(mensaje: String, hilo: Option<u64>, conexion: Option<u64>) -> Registro {
        Registro {
            nivel: NivelRegistro::Advertencia,
            hilo,
            conexion,
            mensaje,
        }
    }

    pub fn error(mensaje: String, hilo: Option<u64>, conexion: Option<u64>) -> Registro {
        Registro {
            nivel: NivelRegistro::Error,
            hilo,
            conexion,
            mensaje,
        }
    }
}

impl ToString for Registro {
    /// Formato: `{Nivel} [hilo: {}] [cliente: {}] {Mensaje}`
    fn to_string(&self) -> String {
        if let Some(hilo) = self.hilo {
            if let Some(conexion) = self.conexion {
                format!(
                    "{} [hilo: {}] [cliente: {}] {}",
                    self.nivel.to_string(),
                    hilo,
                    conexion,
                    self.mensaje
                )
            } else {
                format!(
                    "{} [hilo: {}] {}",
                    self.nivel.to_string(),
                    hilo,
                    self.mensaje
                )
            }
        } else {
            if let Some(conexion) = self.conexion {
                format!(
                    "{} [cliente: {}] {}",
                    self.nivel.to_string(),
                    conexion,
                    self.mensaje
                )
            } else {
                format!("{} {}", self.nivel.to_string(), self.mensaje)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum NivelRegistro {
    Informacion,
    Advertencia,
    Error,
}

impl ToString for NivelRegistro {
    fn to_string(&self) -> String {
        match self {
            NivelRegistro::Informacion => "Info".to_string(),
            NivelRegistro::Advertencia => "Advertencia".to_string(),
            NivelRegistro::Error => "Error".to_string(),
        }
    }
}

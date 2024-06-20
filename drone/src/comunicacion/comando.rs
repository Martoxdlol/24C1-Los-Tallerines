use lib::{incidente::Incidente, serializables::Serializable};

#[derive(Clone, Debug)]

/// Comandos que el dron puede recibir desde la aplicación de monitoreo.
pub enum Comando {
    AtenderIncidente(Incidente),
    DesatenderIncidente(Incidente),
}

impl Serializable for Comando {
    /// Serializa un comando en un vector de bytes.
    fn serializar(&self) -> Vec<u8> {
        match self {
            Comando::AtenderIncidente(incidente) => format!(
                "atender_incidente {}",
                String::from_utf8_lossy(&incidente.serializar())
            )
            .as_bytes()
            .to_vec(),
            Comando::DesatenderIncidente(incidente) => format!(
                "desatender_incidente {}",
                String::from_utf8_lossy(&incidente.serializar())
            )
            .as_bytes()
            .to_vec(),
        }
    }

    /// Deserializa un vector de bytes en un comando.
    fn deserializar(data: &[u8]) -> Result<Self, lib::serializables::error::DeserializationError>
    where
        Self: Sized,
    {
        let texto = String::from_utf8_lossy(data).to_string();
        let primera_palabra = texto.split(' ').next().unwrap_or("");
        let resto_del_texto = texto.split(' ').skip(1).collect::<Vec<&str>>().join(" ");

        if primera_palabra.eq("atender_incidente") {
            if let Ok(incidente) = Incidente::deserializar(resto_del_texto.as_bytes()) {
                return Ok(Comando::AtenderIncidente(incidente));
            }
        }

        if primera_palabra.eq("desasignar_incidente") {
            if let Ok(incidente) = Incidente::deserializar(resto_del_texto.as_bytes()) {
                return Ok(Comando::DesatenderIncidente(incidente));
            }
        } else {
            println!("PRIMERA PALABRA: {}", primera_palabra);
        }

        Err(lib::serializables::error::DeserializationError::InvalidData)
    }
}

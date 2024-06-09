use lib::{
    csv::csv_parsear_linea,
    serializables::{
        deserializador::{self, Deserializador},
        error::DeserializationError,
        guardar::cargar_serializable,
        serializador::Serializador,
        Serializable,
    },
};

#[derive(Debug, Clone)]
pub struct Cuenta {
    pub id: u64,
    pub user: String,
    pub pass: String,
}

impl Cuenta {
    pub fn matches(&self, user: &str, pass: &str) -> bool {
        self.user == user && self.pass == pass
    }

    pub fn cargar(ruta_archivo: &str) -> Result<Vec<Cuenta>, std::io::Error> {
        cargar_serializable(ruta_archivo)
    }
}

impl Serializable for Cuenta {
    fn serializar(&self) -> Vec<u8> {
        let mut serializador = Serializador::new();
        serializador.agregar_elemento(&self.id);
        serializador.agregar_elemento(&self.user);
        serializador.agregar_elemento(&self.pass);
        serializador.bytes
    }

    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError>
    where
        Self: Sized,
    {
        let mut deserializador = Deserializador::new(data.to_vec());

        let id = deserializador.sacar_elemento()?;
        let user = deserializador.sacar_elemento()?;
        let pass = deserializador.sacar_elemento()?;

        Ok(Cuenta { id, user, pass })
    }
}

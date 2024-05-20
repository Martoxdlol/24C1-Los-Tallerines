use self::error::DeserializationError;

pub mod error;
pub mod guardar;

pub trait Serializable {
    fn serializar(&self) -> Vec<u8>;
    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError>
    where
        Self: Sized;
}

// Each element is a line of a csv file
impl<T: Serializable> Serializable for Vec<T> {
    fn serializar(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for element in self {
            data.extend(element.serializar());
            data.push(b'\n');
        }
        data
    }

    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError>
    where
        Self: Sized,
    {
        let texto = String::from_utf8(data.to_vec()).map_err(|_| DeserializationError::InvalidData)?;
        let lineas = texto.lines();

        let mut result = Vec::new();

        for linea in lineas {
            if linea.trim().is_empty() {
                continue;
            }

            let bytes = linea.as_bytes();
            let element = T::deserializar(bytes)?;
            result.push(element);
        }

        Ok(result)
    }
}

pub fn serializar_vec<T: Serializable>(vec: &Vec<T>) -> Vec<u8> {
    vec.serializar()
}

pub fn deserializar_vec<T: Serializable>(data: &[u8]) -> Result<Vec<T>, DeserializationError> {
    Vec::<T>::deserializar(data)
}

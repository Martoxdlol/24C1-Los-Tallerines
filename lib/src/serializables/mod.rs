use self::error::DeserializationError;

pub mod error;

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
        let mut result = Vec::new();
        let mut start = 0;
        for (i, &byte) in data.iter().enumerate() {
            if byte == b'\n' {
                let element = T::deserializar(&data[start..i])?;
                result.push(element);
                start = i + 1;
            }
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
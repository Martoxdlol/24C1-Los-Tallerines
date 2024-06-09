use std::{collections::HashSet, hash::Hash};

use escape::{desescapar_solo_salto_de_linea, escapar_solo_salto_de_linea};

use self::error::DeserializationError;

pub mod deserializador;
pub mod error;
pub mod escape;
pub mod guardar;
pub mod serializador;

pub trait Serializable {
    fn serializar(&self) -> Vec<u8>;
    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError>
    where
        Self: Sized;

    fn serializar_string(&self) -> String {
        String::from_utf8(self.serializar()).unwrap()
    }

    fn deserializar_string(data: &str) -> Result<Self, DeserializationError>
    where
        Self: Sized,
    {
        Self::deserializar(data.as_bytes())
    }
}

// Each element is a line of a csv file
impl<T: Serializable> Serializable for Vec<T> {
    fn serializar(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for element in self {
            data.extend(escapar_solo_salto_de_linea(&element.serializar_string()).as_bytes());
            data.push(b'\n');
        }
        data
    }

    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError>
    where
        Self: Sized,
    {
        let texto =
            String::from_utf8(data.to_vec()).map_err(|_| DeserializationError::InvalidData)?;
        let lineas = texto.lines();

        let mut result = Vec::new();

        for linea in lineas {
            if linea.trim().is_empty() {
                continue;
            }

            let descapado = desescapar_solo_salto_de_linea(&linea);
            let element = T::deserializar(descapado.as_bytes())?;
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

impl Serializable for String {
    fn serializar(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError> {
        String::from_utf8(data.to_vec()).map_err(|_| DeserializationError::InvalidData)
    }
}

impl<T: Serializable + Eq + Hash> Serializable for HashSet<T> {
    fn serializar(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for element in self {
            data.extend(escapar_solo_salto_de_linea(&element.serializar_string()).as_bytes());
            data.push(b'\n');
        }
        data
    }

    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError>
    where
        Self: Sized,
    {
        let texto =
            String::from_utf8(data.to_vec()).map_err(|_| DeserializationError::InvalidData)?;
        let lineas = texto.lines();

        let mut result = HashSet::new();

        for linea in lineas {
            if linea.trim().is_empty() {
                continue;
            }

            let descapado = desescapar_solo_salto_de_linea(&linea);
            let element = T::deserializar(descapado.as_bytes())?;
            result.insert(element);
        }

        Ok(result)
    }
}

impl Serializable for u64 {
    fn serializar(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }

    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError> {
        let texto =
            String::from_utf8(data.to_vec()).map_err(|_| DeserializationError::InvalidData)?;
        let numero = texto
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;
        Ok(numero)
    }
}

impl<T: Serializable> Serializable for Option<T> {
    fn serializar(&self) -> Vec<u8> {
        match self {
            Some(element) => {
                let mut data = Vec::new();
                data.push(b's');
                data.extend(element.serializar());
                data
            }
            None => vec![b'n'],
        }
    }

    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError>
    where
        Self: Sized,
    {
        if data.is_empty() {
            return Err(DeserializationError::InvalidData);
        }

        match data[0] {
            b's' => {
                let element = T::deserializar(&data[1..])?;
                Ok(Some(element))
            }
            b'n' => Ok(None),
            _ => Err(DeserializationError::InvalidData),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestStruct {
        a: u64,
        b: String,
    }

    impl Serializable for TestStruct {
        fn serializar(&self) -> Vec<u8> {
            let mut serializador = serializador::Serializador::new();
            serializador.agregar_elemento(&self.a);
            serializador.agregar_elemento(&self.b);
            serializador.bytes
        }

        fn deserializar(data: &[u8]) -> Result<Self, DeserializationError>
        where
            Self: Sized,
        {
            let mut deserializador = deserializador::Deserializador::new(data.to_vec());
            let a = deserializador.sacar_elemento()?;
            let b = deserializador.sacar_elemento()?;
            Ok(TestStruct { a, b })
        }
    }

    #[test]
    fn serializar_vec() {
        let vec = vec![
            TestStruct {
                a: 1,
                b: "hola".to_string(),
            },
            TestStruct {
                a: 2,
                b: "chau".to_string(),
            },
        ];

        let serializado = vec.serializar();
        let deserializado = deserializar_vec::<TestStruct>(&serializado).unwrap();

        assert_eq!(vec, deserializado);
    }

    #[test]
    fn serializar_option() {
        let some = Some(TestStruct {
            a: 1,
            b: "hola".to_string(),
        });

        let none: Option<TestStruct> = None;

        let serializado_some = some.serializar();
        let serializado_none = none.serializar();

        let deserializado_some = Option::<TestStruct>::deserializar(&serializado_some).unwrap();
        let deserializado_none = Option::<TestStruct>::deserializar(&serializado_none).unwrap();

        assert_eq!(some, deserializado_some);
        assert_eq!(none, deserializado_none);
    }
}

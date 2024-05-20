use std::fs;

use super::Serializable;

pub fn guardar_serializable<T: Serializable>(
    serializable: &T,
    path: &str,
) -> Result<(), std::io::Error> {
    let data = serializable.serializar();

    fs::write(path, data)
}

pub fn cargar_serializable<T: Serializable>(path: &str) -> Result<T, std::io::Error> {
    let data = fs::read(path)?;

    T::deserializar(&data).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Error deserializando datos",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serializables::error::DeserializationError;

    #[derive(Debug, PartialEq)]
    struct TestStruct {
        a: u32,
        b: String,
    }

    impl Serializable for TestStruct {
        fn serializar(&self) -> Vec<u8> {
            let mut data = Vec::new();
            data.extend(self.a.to_be_bytes().iter());
            data.extend(self.b.as_bytes());
            data
        }

        fn deserializar(data: &[u8]) -> Result<Self, DeserializationError> {
            if data.len() < 4 {
                return Err(DeserializationError::InvalidData);
            }

            let a = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
            let b = String::from_utf8(data[4..].to_vec())
                .map_err(|_| DeserializationError::InvalidData)?;

            Ok(TestStruct { a, b })
        }
    }

    #[test]
    fn test_guardar_cargar_serializable() {
        let test_struct = TestStruct {
            a: 42,
            b: "Hello, world!".to_string(),
        };

        guardar_serializable(&test_struct, "/tmp/serializable.test.dat").unwrap();

        let loaded_struct =
            cargar_serializable::<TestStruct>("/tmp/serializable.test.dat").unwrap();

        assert_eq!(test_struct, loaded_struct);
    }

    #[test]
    fn test_guardar_cargar_vector_de_serializables() {
        let test_structs = vec![
            TestStruct {
                a: 42,
                b: "Hello, world!".to_string(),
            },
            TestStruct {
                a: 1337,
                b: "Goodbye, world!".to_string(),
            },
        ];

        guardar_serializable(&test_structs, "/tmp/serializable.test.dat").unwrap();

        let loaded_structs =
            cargar_serializable::<Vec<TestStruct>>("/tmp/serializable.test.dat").unwrap();

        assert_eq!(test_structs, loaded_structs);
    }
}

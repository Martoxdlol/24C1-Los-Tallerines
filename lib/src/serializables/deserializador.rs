use std::str::FromStr;

use super::{error::DeserializationError, escape::desescapar, Serializable};

pub struct Deserializador {
    pub bytes: Vec<u8>,
    puntero: usize,
}

impl Deserializador {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes, puntero: 0 }
    }

    pub fn sacar_elemento<T: FromStr>(&mut self) -> Result<T, DeserializationError> {
        let mut escapado = false;
        let mut resultado = Vec::new();

        for &byte in self.bytes.iter().skip(self.puntero) {
            self.puntero += 1;

            if escapado {
                escapado = false;
                resultado.push(byte);
                continue;
            }

            match byte {
                b'\\' => {
                    escapado = true;
                    resultado.push(byte)
                }
                b',' => break,
                _ => resultado.push(byte),
            }
        }

        desescapar(&String::from_utf8_lossy(&resultado))
            .parse()
            .map_err(|_| DeserializationError::InvalidData)
    }

    pub fn sacar_elemento_serializable<T: Serializable>(
        &mut self,
    ) -> Result<T, DeserializationError> {
        let texto = self.sacar_elemento::<String>()?;

        T::deserializar(&texto.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializar() {
        let mut deserializador = Deserializador::new(b"hola,mundo".to_vec());

        assert_eq!(deserializador.sacar_elemento::<String>().unwrap(), "hola");
        assert_eq!(deserializador.sacar_elemento::<String>().unwrap(), "mundo");
    }

    #[test]
    fn deserializar_especial() {
        let mut deserializador = Deserializador::new(b"hol\\,a,mundo".to_vec());

        assert_eq!(deserializador.sacar_elemento::<String>().unwrap(), "hol,a");
        assert_eq!(deserializador.sacar_elemento::<String>().unwrap(), "mundo");
    }

    #[test]
    fn deserializar_vector() {
        let mut deserializador = Deserializador::new(b"1\\n2\\n".to_vec());

        let vector: Vec<String> = deserializador.sacar_elemento_serializable().unwrap();
        assert_eq!(vector, vec!["1".to_string(), "2".to_string()]);
    }

    #[test]
    fn deserializar_vector_especial() {
        let mut deserializador = Deserializador::new(b"hol\\,a\\nmundo\\n".to_vec());

        let vector: Vec<String> = deserializador.sacar_elemento_serializable().unwrap();
        assert_eq!(vector, vec!["hol,a".to_string(), "mundo".to_string()]);
    }
}

use super::{escape::escapar, Serializable};

pub struct Serializador {
    pub bytes: Vec<u8>,
    pub elementos: usize,
}

impl Serializador {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            elementos: 0,
        }
    }

    pub fn agregar_elemento<T: ToString>(&mut self, elemento: &T) {
        self.agregar_elemento_serializable(&elemento.to_string());
    }

    pub fn agregar_elemento_serializable<T: Serializable>(&mut self, elemento: &T) {
        if self.elementos > 0 {
            self.bytes.push(b',');
        }
        self.elementos += 1;

        self.bytes
            .extend(escapar(&elemento.serializar_string()).as_bytes());
    }

    pub fn obtener_texto(&self) -> String {
        String::from_utf8_lossy(&self.bytes).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializar() {
        let mut serializador = Serializador::new();
        serializador.agregar_elemento(&"hola");
        serializador.agregar_elemento(&"mundo");

        assert_eq!(serializador.bytes, b"hola,mundo");
    }

    #[test]
    fn serializar_especial() {
        let mut serializador = Serializador::new();
        serializador.agregar_elemento(&"hol,a");
        serializador.agregar_elemento(&"mundo");

        assert_eq!(serializador.obtener_texto(), "hol\\,a,mundo".to_string());
    }

    #[test]
    fn serializar_vector() {
        let mut serializador = Serializador::new();
        serializador.agregar_elemento_serializable(&vec!["1".to_string(), "2".to_string()]);

        assert_eq!(serializador.obtener_texto(), "1\\n2\\n".to_string());
    }

    #[test]
    fn serializar_vector_especial() {
        let mut serializador = Serializador::new();
        serializador.agregar_elemento_serializable(&vec!["hol,a".to_string(), "mundo".to_string()]);

        assert_eq!(
            serializador.obtener_texto(),
            "hol\\,a\\nmundo\\n".to_string()
        );
    }
}

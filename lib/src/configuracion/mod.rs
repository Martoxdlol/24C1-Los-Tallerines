use std::{collections::HashMap, io};

pub struct ArchivoConfiguracion {
    valores: HashMap<String, String>,
}

impl ArchivoConfiguracion {
    pub fn new() -> Self {
        ArchivoConfiguracion {
            valores: HashMap::new(),
        }
    }

    pub fn leer(ruta: &str) -> io::Result<Self> {
        // Leer archivo de configuración
        let contenido = std::fs::read_to_string(ruta)?;

        Ok(Self::parsear(&contenido))
    }

    pub fn obtener<T: std::str::FromStr>(&self, clave: &str) -> Option<T> {
        self.valores.get(clave).and_then(|v| v.parse().ok())
    }

    pub fn setear<T: std::string::ToString>(&mut self, clave: &str, valor: T) {
        self.valores.insert(clave.to_string(), valor.to_string());
    }

    pub fn parsear(texto: &str) -> Self {
        let mut config = ArchivoConfiguracion::new();

        let lineas = texto.lines();

        for line in lineas {
            let mut partes = line.split('=');
            let clave = partes.next();
            let valor = partes.next();

            if let (Some(clave), Some(valor)) = (clave, valor) {
                config.setear(clave, Self::parsear_valor(valor));
            }
        }

        config
    }

    fn parsear_valor(valor: &str) -> String {
        let valor_trim = valor.trim();

        if valor_trim.starts_with('"') && valor_trim.ends_with('"') {
            valor_trim[1..valor_trim.len() - 1].to_string()
        } else {
            valor_trim.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parsear_valor() {
        let valor = super::ArchivoConfiguracion::parsear_valor("\"hola\"");
        assert_eq!(valor, "hola");
    }

    #[test]
    fn parsear() {
        let texto = "clave1=valor1\nclave2=\"valor2\"";
        let config = super::ArchivoConfiguracion::parsear(texto);

        assert_eq!(config.obtener::<String>("clave1"), Some("valor1".to_string()));
        assert_eq!(config.obtener::<String>("clave2"), Some("valor2".to_string()));
    }

    #[test]
    fn leer() {
        let texto = "clave1=valor1\nclave2=\"valor2\"";
        std::fs::write("config.txt", texto).unwrap();

        let config = super::ArchivoConfiguracion::leer("config.txt").unwrap();

        assert_eq!(config.obtener::<String>("clave1"), Some("valor1".to_string()));
        assert_eq!(config.obtener::<String>("clave2"), Some("valor2".to_string()));

        std::fs::remove_file("config.txt").unwrap();
    }

    #[test]
    fn setear() {
        let mut config = super::ArchivoConfiguracion::new();
        config.setear("clave1", "valor1");

        assert_eq!(config.obtener::<String>("clave1"), Some("valor1".to_string()));
    }

    #[test]
    fn obtener() {
        let mut config = super::ArchivoConfiguracion::new();
        config.setear("clave1", "valor1");

        assert_eq!(config.obtener::<String>("clave1"), Some("valor1".to_string()));
        assert_eq!(config.obtener::<i32>("clave1"), None);
    }

    #[test]
    fn obtener_bool() {
        let mut config = super::ArchivoConfiguracion::new();
        config.setear("clave1", "true");

        assert_eq!(config.obtener::<bool>("clave1"), Some(true));
    }

    #[test]
    fn obtener_float() {
        let mut config = super::ArchivoConfiguracion::new();
        config.setear("clave1", "3.14");

        assert_eq!(config.obtener::<f32>("clave1"), Some(3.14));
    }
}
pub fn escapar(texto: &str) -> String {
    let mut resultado = String::new();

    for caracter in texto.chars() {
        match caracter {
            '\n' => resultado.push_str("\\n"),
            '\r' => resultado.push_str("\\r"),
            '\t' => resultado.push_str("\\t"),
            '\\' => resultado.push_str("\\\\"),
            '"' => resultado.push_str("\\\""),
            ',' => resultado.push_str("\\,"),
            _ => resultado.push(caracter),
        }
    }

    resultado
}

pub fn desescapar(texto: &str) -> String {
    let mut resultado = String::new();
    let mut iter = texto.chars().peekable();

    while let Some(caracter) = iter.next() {
        if caracter == '\\' {
            match iter.peek() {
                Some(&'n') => {
                    resultado.push('\n');
                    iter.next();
                }
                Some(&'r') => {
                    resultado.push('\r');
                    iter.next();
                }
                Some(&'t') => {
                    resultado.push('\t');
                    iter.next();
                }
                Some(&'\\') => {
                    resultado.push('\\');
                    iter.next();
                }
                Some(&'"') => {
                    resultado.push('"');
                    iter.next();
                }
                Some(&',') => {
                    resultado.push(',');
                    iter.next();
                }
                _ => resultado.push(caracter),
            }
        } else {
            resultado.push(caracter);
        }
    }

    resultado
}

pub fn escapar_solo_salto_de_linea(texto: &str) -> String {
    let mut resultado = String::new();

    for caracter in texto.chars() {
        match caracter {
            '\n' => resultado.push_str("\\n"),
            _ => resultado.push(caracter),
        }
    }

    resultado
}

pub fn desescapar_solo_salto_de_linea(texto: &str) -> String {
    let mut resultado = String::new();
    let mut iter = texto.chars().peekable();

    while let Some(caracter) = iter.next() {
        if caracter == '\\' {
            match iter.peek() {
                Some(&'n') => {
                    resultado.push('\n');
                    iter.next();
                }
                _ => resultado.push(caracter),
            }
        } else {
            resultado.push(caracter);
        }
    }

    resultado
}

#[cfg(test)]
mod tests {
    #[test]
    fn escapar() {
        let texto = "hola\nmundo";
        let resultado = super::escapar(texto);
        assert_eq!(resultado, "hola\\nmundo");
    }

    #[test]
    fn desescapar() {
        let texto = "hola\\nmundo";
        let resultado = super::desescapar(texto);
        assert_eq!(resultado, "hola\nmundo");
    }

    #[test]
    fn coma() {
        let texto = "hola,mundo";
        let resultado = super::escapar(texto);
        assert_eq!(resultado, "hola\\,mundo");

        let texto = "hola\\,mundo";
        let resultado = super::desescapar(texto);
        assert_eq!(resultado, "hola,mundo");
    }

    #[test]
    fn barra_y_coma() {
        let texto = "hola\\,mundo";
        let resultado = super::escapar(texto);
        assert_eq!(resultado, "hola\\\\\\,mundo");

        let texto = "hola\\\\\\,mundo";
        let resultado = super::desescapar(texto);
        assert_eq!(resultado, "hola\\,mundo");
    }
}

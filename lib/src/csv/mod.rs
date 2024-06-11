/// Módulo para manejar archivos CSV
///
/// parsea la linea de un archivo CSV y la convierte en un vector de strings
pub fn csv_parsear_linea(linea: &str) -> Vec<String> {
    linea.split(',').map(|s| s.to_string()).collect()
}

/// Codifica un vector de strings en una linea de un archivo CSV
pub fn csv_encodear_linea(linea: &[String]) -> String {
    linea.join(",")
}

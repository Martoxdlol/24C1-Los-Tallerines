pub fn csv_parsear_linea(linea: &str) -> Vec<String> {
    linea.split(",").map(|s| s.to_string()).collect()
}

pub fn csv_encodear_linea(linea: &[String]) -> String {
    linea.join(",")
}

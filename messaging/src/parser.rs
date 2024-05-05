use super::{message::Message, resultado_linea::ResultadoLinea};

pub struct Parser {
    // Bytes que fue acumulando que todavía no se pudieron convertir en ninguna estructura
    bytes_pendientes: Vec<u8>,
    /// Se utiliza internamente para llevar el estado del parseo en diferentes casos
    continuar_en_indice: usize,
    /// La primera linea del mensaje que se está parseando (ejemplo: se encontró un PUB y falta leer el payload)
    actual: Option<ResultadoLinea>,
    /// Los headers del mensaje que se está parseando ahora
    headers: Option<Vec<u8>>,
}

/// La responsabilidad del parser es recibir bytes de la conexión y tranformarlos a mensajes
///
/// Acumula los bytes que recibe y cuando tiene suficientes bytes para formar un mensaje
/// lo parsea y lo devuelve
///
/// El proceso de utilizar los bytes que fue recibiendo y convertirlos a mensajes se llama
/// desde la función `proximo_mensaje`, esta te entrega mensajes uno por uno
///
/// El parser se encarga de ir liberando los bytes que ya utilizó y de tener en cuenta los estados intermedios
/// (es decir, si le llega un mensaje que no está completo, lo guarda y espera a que lleguen más bytes)
impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub fn new() -> Self {
        Self {
            bytes_pendientes: Vec::new(),
            continuar_en_indice: 0,
            actual: None,
            headers: None,
        }
    }

    /// Agrega bytes al parser
    pub fn agregar_bytes(&mut self, bytes: &[u8]) {
        self.bytes_pendientes.extend_from_slice(bytes);
    }

    /// Devuelve la próxima línea que se encuentra en los bytes que se le pasaron
    /// O `None` si no se encontró ninguna línea (porque no se recibieron suficientes bytes)
    pub fn proxima_linea(&mut self) -> Option<String> {
        let mut last_char_cr = false;

        // Pasar por todos los bytes que tenemos que todavía no procesamos
        // Estamos buscando un salto de linea, este se marca como \r\n
        for i in self.continuar_en_indice..self.bytes_pendientes.len() {
            // Si encontramos un \r, marcamos que el último caracter fue un \r
            if self.bytes_pendientes[i] == b'\r' { // La b es para que lo tome como binario
                last_char_cr = true;
            // Si encontramos un \n y el último caracter fue un \r, encontramos un mensaje (o al menos la primera linea)
            } else if last_char_cr && self.bytes_pendientes[i] == b'\n' {
                self.continuar_en_indice = i + 1;

                let result = String::from_utf8_lossy(&self.bytes_pendientes[..self.continuar_en_indice])
                    .trim_end() // Elimina los espacios vacios al final
                    .to_string();

                self.resetear_bytes();
                return Some(result);
            } else {
                last_char_cr = false;
            }
        }

        None
    }

    pub fn proximo_mensaje(&mut self) -> Option<Message> {
        // Si actualmente se está parseando un PUB buscamos el payload
        if let Some(ResultadoLinea::Pub(topic, reply_to, total_bytes)) = &self.actual {
            // No hay suficientes bytes para el payload
            if self.bytes_pendientes.len() < *total_bytes {
                return None;
            }

            self.continuar_en_indice = *total_bytes;

            // Si hay suficientes bytes para el payload, devolvemos el mensaje
            let resultado = Some(Message::Pub(
                topic.to_string(),
                reply_to.clone(),
                self.bytes_pendientes[..*total_bytes].to_vec(),
            ));

            self.resetear_todo();

            return resultado;
        }

        // Si actualmente se está parseando un HPUB buscamos el payload
        if let Some(ResultadoLinea::Hpub(topic, reply_to, headers_bytes, total_bytes)) =
            &self.actual
        {
            // Si ya habíamos encontrado los headers antes,
            // tenemos todo para buscar el payload y si está completo devolver el mensaje
            if let Some(headers) = &self.headers {
                // No hay suficientes bytes para el payload
                if self.bytes_pendientes.len() < *total_bytes {
                    return None;
                }

                self.continuar_en_indice = *total_bytes;

                // Si hay suficientes bytes para el payload, devolvemos el mensaje
                let resultado = Some(Message::Hpub(
                    topic.to_string(),
                    reply_to.clone(),
                    headers.clone(),
                    self.bytes_pendientes[..*total_bytes].to_vec(),
                ));

                self.resetear_todo();

                return resultado;
            } else {
                // Si no encontramos los headers antes, buscamos los headers
                if self.bytes_pendientes.len() < *headers_bytes {
                    return None;
                }

                self.headers = Some(self.bytes_pendientes[..*headers_bytes].to_vec());
                self.continuar_en_indice = *headers_bytes;
                return None;
            }
        }

        // Si actualmente no se está parseando nada, buscamos la próxima línea
        if self.actual.is_none() {
            let linea = self.proxima_linea()?;

            match self.parsear_linea(&linea) {
                ResultadoLinea::Hpub(subject, reply_to, header_bytes, total_bytes) => {
                    self.actual = Some(ResultadoLinea::Hpub(
                        subject,
                        reply_to,
                        header_bytes,
                        total_bytes,
                    ));
                }
                ResultadoLinea::MensajeIncorrecto => {
                    return Some(Message::Err("Mensaje incorrecto".to_string()));
                }
                ResultadoLinea::StringVacio => {
                    return self.proximo_mensaje();
                }
                ResultadoLinea::Sub(subject, queue_group, sid) => {
                    return Some(Message::Sub(subject, queue_group, sid));
                }
                ResultadoLinea::Unsub(sid, max_mgs) => {
                    return Some(Message::Unsub(sid, max_mgs));
                }
                ResultadoLinea::Pub(subject, reply_to, bytes) => {
                    self.actual = Some(ResultadoLinea::Pub(subject, reply_to, bytes));
                }
                ResultadoLinea::Ping => {
                    return Some(Message::Ping());
                }
                ResultadoLinea::Connect => {
                    return Some(Message::Connect("".to_string()));
                }
            }
        }
        None
    }

    fn parsear_linea(&self, linea: &str) -> ResultadoLinea {
        let palabras = linea
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let palabra_vacia = "".to_string();
        let primera_palabra = palabras.first().unwrap_or(&palabra_vacia).to_lowercase();

        if primera_palabra.eq("pub") {
            return self.linea_pub(&palabras[1..]);
        }

        if primera_palabra.eq("hpub") {
            return self.linea_hpub(&palabras[1..]);
        }

        if primera_palabra.eq("sub") {
            return self.linea_sub(&palabras[1..]);
        }

        if primera_palabra.eq("unsub") {
            return self.linea_unsub(&palabras[1..]);
        }

        if primera_palabra.eq("") {
            return ResultadoLinea::StringVacio;
        }
        if primera_palabra.eq("ping") {
            return ResultadoLinea::Ping;
        }
        if primera_palabra.eq("connect") {
            return ResultadoLinea::Connect;
        }

        ResultadoLinea::MensajeIncorrecto
    }

    fn linea_pub(&self, palabras: &[String]) -> ResultadoLinea {
        // Buscamos si es de 2 o 3 para saber si tiene reply_to
        if palabras.len() == 2 {
            let bytes = match palabras[1].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };

            return ResultadoLinea::Pub(palabras[0].to_string(), None, bytes);
        }

        if palabras.len() == 3 {
            let bytes = match palabras[2].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };

            return ResultadoLinea::Pub(
                palabras[0].to_string(),
                Some(palabras[1].to_string()),
                bytes,
            );
        }

        ResultadoLinea::MensajeIncorrecto
    }

    fn linea_hpub(&self, palabras: &[String]) -> ResultadoLinea {
        // Buscamos si es de 3 o 4 para saber si tiene reply_to
        if palabras.len() == 3 {
            let bytes = match palabras[1].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };
            let headers_bytes = match palabras[2].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };

            return ResultadoLinea::Hpub(palabras[0].to_string(), None, headers_bytes, bytes);
        }

        if palabras.len() == 4 {
            let bytes = match palabras[2].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };
            let headers_bytes = match palabras[3].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };

            return ResultadoLinea::Hpub(
                palabras[0].to_string(),
                Some(palabras[1].to_string()),
                headers_bytes,
                bytes,
            );
        }

        ResultadoLinea::MensajeIncorrecto
    }

    fn linea_sub(&self, palabras: &[String]) -> ResultadoLinea {
        if palabras.len() == 2 {
            let subject = &palabras[0];
            let sid = &palabras[1];
            return ResultadoLinea::Sub(subject.to_string(), None, sid.to_string());
        }

        if palabras.len() == 3 {
            let subject = &palabras[1];
            let queue_group = &palabras[2];
            let sid = &palabras[3];
    
            return ResultadoLinea::Sub(
                subject.to_string(),
                Some(queue_group.to_string()),
                sid.to_string(),
            )
        }

        ResultadoLinea::MensajeIncorrecto
    }

    fn linea_unsub(&self, palabras: &[String]) -> ResultadoLinea {
        if palabras.len() != 1 {
            return ResultadoLinea::MensajeIncorrecto;
        }

        let sid = &palabras[0];
        let max_msgs = palabras.get(2).map(|s| s.parse::<u64>().unwrap());

        ResultadoLinea::Unsub(sid.to_string(), max_msgs)
    }

    /// Libera los bytes de la parte del mensaje que ya se parseó
    ///
    /// Por ejemplo:
    /// - La primera linea de cualquier mensaje
    /// - El payload
    /// - Los headers
    fn resetear_bytes(&mut self) {
        self.bytes_pendientes.drain(..self.continuar_en_indice);
        self.continuar_en_indice = 0;
    }

    fn resetear_todo(&mut self) {
        self.resetear_bytes();
        self.actual = None;
        self.headers = None;
    }
}

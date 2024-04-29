enum Segmento {
    Texto(String),
    Asteriso,
}

pub struct Subject {
    patron: Vec<Segmento>,
    exacto: bool,
}

impl Subject {
    pub fn new(patron: String) -> Result<Self, ()> {
        let mut segmentos = Vec::new();
        let mut exacto = true;

        for str in patron.split(".") {
            if !exacto {
                return Err(());
            }

            if str.eq("*") {
                segmentos.push(Segmento::Asteriso);
            } else if str.eq(">") {
                exacto = false;
            } else {
                segmentos.push(Segmento::Texto(str.to_string()));
            }
        }

        return Ok(Self {
            patron: segmentos,
            exacto,
        });
    }

    pub fn test(&self, subject: &str) -> bool {
        let segmentos = subject.split(".").collect::<Vec<&str>>();
        if self.patron.len() > segmentos.len() {
            return false;
        }

        if self.exacto && segmentos.len() != self.patron.len() {
            return false;
        }

        let mut i = 0;
        for segmento in segmentos.iter() {
            let segmento_patron = &self.patron[i];

            if let Segmento::Texto(t) = segmento_patron {
                if !t.eq(segmento) {
                    return false;
                }
            }
            i += 1;
        }

        return true;
    }
}

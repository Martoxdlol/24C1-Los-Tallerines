use crate::mensaje::Mensaje;

struct Subscripcion {
    id: String,
}

impl Subscripcion {
    pub fn leer(&self) -> Option<Mensaje> {
        todo!();
    }
}

impl Drop for Subscripcion {
    fn drop(&mut self) {
        todo!();
    }
}

impl Iterator for Subscripcion {
    type Item = Mensaje;

    fn next(&mut self) -> Option<Self::Item> {
        self.leer()
    }
}

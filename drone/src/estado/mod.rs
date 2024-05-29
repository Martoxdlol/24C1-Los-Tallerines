use std::collections::{HashMap, HashSet};

use lib::{dron::Dron, incidente::Incidente};

use std::{
    sync::mpsc,
    thread,
};

#[derive(Debug)]
/// Estado de la aplicación del dron
pub struct Estado {
    /// Dron de la aplicación
    pub dron: Option<Dron>,
    /// Incidentes activos
    pub incidentes: HashMap<u64, Incidente>,
}

impl Default for Estado {
    fn default() -> Self {
        Self::new()
    }
}

impl Estado {
    pub fn new() -> Self {
        Self {
            dron: None,
            incidentes: HashMap::new(),
        }
    }

    pub fn iniciar_dron(&mut self, mut dron: Dron) {
        dron.incidentes_cercanos = self.incidentes_en_rango(&dron);
        // REVISAR PORQUE ACÁ SE MUESTRAN LOS INCIDENTES EN RANGO PERO AFUERA DE ESTA FUNCION NO
        println!("\nIncidentes en rango: {:?}", dron.incidentes_cercanos);
    }

    pub fn iniciar_bateria_dron(&mut self, mut dron: Dron) {
        let (tx, rx) = mpsc::channel::<u64>();

        let hilo_bateria = thread::spawn(move || {
        });
    }

    /// Incidentes que están en el rango de un dron
    pub fn incidentes_en_rango(&self, dron: &Dron) -> HashSet<u64> {
        //let mut incidentes: HashSet<u64> = HashSet::new();
        let mut incidentes: HashSet<u64> = HashSet::from([20]);
        for incidente in self.incidentes.values() {
            if incidente.posicion().distancia(&dron.posicion()) < dron.rango {
                incidentes.insert(incidente.id);
            }
        }
        incidentes
    }
}
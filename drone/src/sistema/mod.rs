use lib::{
    coordenadas::Coordenadas,
    dron::{accion::Accion, Dron},
};

use crate::comunicacion::Comunicacion;

pub struct Sistema {
    dron: Dron,
    ms_ultima_iteracion: i64,
    diferencial_tiempo: f64,
    comunicacion: Comunicacion,
}

impl Sistema {
    pub fn new(dron: Dron, comunicacion: Comunicacion) -> Self {
        Self {
            dron,
            comunicacion,
            ms_ultima_iteracion: 0,
            diferencial_tiempo: 0.,
        }
    }

    /// Devuelve el tiempo en segundos desde la última iteración (en general 0.100s +-)
    fn establecer_diferencial_tiempo(&mut self) {
        let ms_ahora = chrono::offset::Local::now().timestamp_millis();
        let diferencial_tiempo = ms_ahora - self.ms_ultima_iteracion;
        self.diferencial_tiempo = (diferencial_tiempo as f64) / 1000.;
    }

    pub fn iniciar(&mut self) {
        self.ms_ultima_iteracion = chrono::offset::Local::now().timestamp_millis();
        loop {
            self.ciclo();
            self.ms_ultima_iteracion = chrono::offset::Local::now().timestamp_millis();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn ciclo(&mut self) {
        self.establecer_diferencial_tiempo();
        self.descarga_bateria();
        self.mover();

        if self.en_destino() {
            self.dron.velocidad_actual = 0.;

            if let Accion::Cargar = self.dron.accion() {
                self.dron.bateria_actual = 100.;
            }
        } else {
            self.establecer_velocidad();
            self.establecer_direccion();
        }

        self.comunicacion.ciclo(&mut self.dron);
    }

    fn destino(&self) -> Coordenadas {
        self.dron.destino()
    }

    fn en_destino(&self) -> bool {
        self.destino().distancia(&self.dron.posicion) < 1.
    }

    fn establecer_direccion(&mut self) {
        let diff_lat = self.destino().lat - self.dron.posicion.lat;
        let diff_lon = self.destino().lon - self.dron.posicion.lon;
        let hipotenusa = (diff_lat.powi(2) + diff_lon.powi(2)).sqrt();
        let direccion = (diff_lat / hipotenusa).acos().to_degrees();

        if diff_lon < 0. {
            self.dron.direccion_actual = 360. - direccion;
        } else {
            self.dron.direccion_actual = direccion;
        }
    }

    fn establecer_velocidad(&mut self) {
        let distancia = self.destino().distancia(&self.dron.posicion);
        if distancia > 20. {
            self.dron.velocidad_actual = self.dron.velocidad_maxima;
        } else {
            self.dron.velocidad_actual = self.dron.velocidad_maxima * 0.4;
        }
    }

    fn descarga_bateria(&mut self) {
        self.dron.bateria_actual -= self.dron.velocidad_descarga_bateria * self.diferencial_tiempo;
    }

    fn mover(&mut self) {
        self.dron.posicion = self.dron.posicion.mover_en_direccion(
            self.diferencial_tiempo * self.dron.velocidad_actual,
            self.dron.direccion_actual,
        );
    }
}

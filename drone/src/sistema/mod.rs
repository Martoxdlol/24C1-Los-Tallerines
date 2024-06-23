use lib::{
    coordenadas::Coordenadas,
    dron::{accion::Accion, Dron},
};

use crate::comunicacion::Comunicacion;

/// Sistema interno del dron
///
/// Se encarga de mover el dron, mantener la batería y comunicarse con el servidor
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

    /// Inicia el sistema del dron
    pub fn iniciar(&mut self) {
        self.ms_ultima_iteracion = chrono::offset::Local::now().timestamp_millis();
        loop {
            self.ciclo();
            self.ms_ultima_iteracion = chrono::offset::Local::now().timestamp_millis();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    /// Ciclo del dron. Cambia la velocidad o la bateria de ser necesario.
    fn ciclo(&mut self) {
        self.establecer_diferencial_tiempo();
        self.descarga_bateria();
        self.mover();

        if let Accion::Cargar = self.dron.accion() {
            self.dron.incidente_actual.take();
        }

        // Si el dron llego, frena (velocidad = 0)
        if self.en_destino() {
            self.dron.velocidad_actual = 0.;

            // Si el dron esta en el centro de carga, se recarga
            if let Accion::Cargar = self.dron.accion() {
                if self.en_destino() {
                    self.dron.bateria_actual = 100.;
                }
            }
        } else {
            self.establecer_velocidad();
            self.establecer_direccion();
        }

        self.comunicacion.ciclo(&mut self.dron);
    }

    /// Devuelve a donde tiene que ir el dron.
    fn destino(&self) -> Coordenadas {
        self.dron.destino()
    }

    /// Devuelve si el dron esta en su destino.
    fn en_destino(&self) -> bool {
        self.destino().distancia(&self.dron.posicion) < 1.
    }

    /// Establece la dirección del dron
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

    /// Establece la velocidad del dron.
    ///
    /// Esta varía según la distancia al destino. A medida que llega disminuye la velocidad
    /// (Disimular que esta frenando)
    fn establecer_velocidad(&mut self) {
        let distancia = self.destino().distancia(&self.dron.posicion);
        if distancia > 20. {
            self.dron.velocidad_actual = self.dron.velocidad_maxima;
        } else {
            self.dron.velocidad_actual = self.dron.velocidad_maxima * 0.4;
        }
    }

    /// Descarga la batería del dron
    fn descarga_bateria(&mut self) {
        self.dron.bateria_actual -= self.dron.velocidad_descarga_bateria * self.diferencial_tiempo;
    }

    /// Mueve el dron
    fn mover(&mut self) {
        self.dron.posicion = self.dron.posicion.mover_en_direccion(
            self.diferencial_tiempo * self.dron.velocidad_actual,
            self.dron.direccion_actual,
        );
    }
}

#[cfg(test)]

mod tests {
    use lib::{configuracion::Configuracion, coordenadas::Coordenadas, dron::Dron};

    use crate::comunicacion::Comunicacion;

    use super::Sistema;

    #[test]
    fn esta_en_destino() {
        let dron = Dron {
            id: 1,
            rango: 1500.0,
            central_de_carga: Coordenadas::from_lat_lon(-34.6079162126949, -58.40631119706255),
            punto_de_espera: Coordenadas::from_lat_lon(1.0, 1.0),
            velocidad_maxima: 10.0,
            velocidad_descarga_bateria: 1.0 / 3600.0,
            posicion: Coordenadas::from_lat_lon(-34.6079162126949, -58.40631119706255),
            direccion_actual: 0.0,
            bateria_actual: 100.0,
            velocidad_actual: 0.0,
            incidente_actual: None,
            envio_ultimo_estado: 0,
        };
        let mut configuracion = Configuracion::new();
        configuracion.setear("user", "user");
        configuracion.setear("pass", "pass");

        let comunicacion = Comunicacion::new(&configuracion);
        let mut sistema = Sistema::new(dron, comunicacion);

        sistema.ciclo();
        assert_eq!(sistema.dron.velocidad_actual, 0.0); // Si la velocidad es 0, esta en destino
    }

    #[test]
    fn esta_cargando() {
        let dron = Dron {
            id: 1,
            rango: 1500.0,
            central_de_carga: Coordenadas::from_lat_lon(-34.6079162126949, -58.40631119706255),
            punto_de_espera: Coordenadas::from_lat_lon(1.0, 1.0),
            velocidad_maxima: 10.0,
            velocidad_descarga_bateria: 1.0 / 3600.0,
            posicion: Coordenadas::from_lat_lon(-34.6079162126949, -58.40631119706255),
            direccion_actual: 0.0,
            bateria_actual: 5.0,
            velocidad_actual: 0.0,
            incidente_actual: None,
            envio_ultimo_estado: 0,
        };

        let mut configuracion = Configuracion::new();
        configuracion.setear("user", "user");
        configuracion.setear("pass", "pass");

        let comunicacion = Comunicacion::new(&configuracion);
        let mut sistema = Sistema::new(dron, comunicacion);

        sistema.ciclo();
        assert_eq!(sistema.dron.bateria_actual, 100.0); // Si la bateria es 100, la bateria cargo porque esta en su centro de carga
    }
}

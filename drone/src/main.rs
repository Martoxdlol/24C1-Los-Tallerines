// use messaging::client::NATSClient;
use chrono::prelude::*;
use lib::{coordenadas::Coordenadas, dron::Dron};

fn main() {
    let mut dron = Dron::new(-34.6037, -58.3816, 100., 155.);

    let ms_ultima_iteracion = chrono::offset::Local::now().timestamp_millis();

    let descarga_bateria = 1. / 3600.;

    loop {
        let ms_ahora = chrono::offset::Local::now().timestamp_millis();
        let diferencial_tiempo = ms_ahora - ms_ultima_iteracion;

        let movimiento = dron.rapidez * diferencial_tiempo as f64;

        let posicion = Coordenadas::from_lat_lon(dron.lat, dron.lon)
            .mover_en_direccion(movimiento, dron.direccion);

        let distancia_a_destino = posicion.distancia(&dron.destino);

        let bateria = dron.bateria - descarga_bateria;

        if bateria < 10. {
            dron.destino = Coordenadas::from_lat_lon(-34.6040, -58.3619);
        }

        if distancia_a_destino < 1. {
            // Llegamos al destino yey!

            // si destino es estación de carga, cargar bateria

            dron.rapidez = 0.;
        }

        if distancia_a_destino > 1. {
            // Ajustar direccion
            let diff_lat = dron.destino.lat - posicion.lat;
            let diff_lon = dron.destino.lon - posicion.lon;
            let hipotenusa = (diff_lat.powi(2) + diff_lon.powi(2)).sqrt();
            let direccion = (diff_lat / hipotenusa).acos().to_degrees();
        }

        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

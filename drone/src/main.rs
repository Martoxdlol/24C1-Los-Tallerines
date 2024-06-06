// use messaging::client::NATSClient;
use chrono::prelude::*;
use lib::coordenadas::Coordenadas;

fn main() {

    let mut bateria = 100.;

    let mut ms_ultima_iteracion = chrono::offset::Local::now().timestamp_millis();

    let mut rapidez = 2.5; // m/s

    let mut direccion = 155.; // grados

    let mut posicion = Coordenadas::from_lat_lon(-34.6037, -58.3816);

    let mut destino = Coordenadas::from_lat_lon(-34.6020, -58.3689);

    let descarga_bateria = 1./3600.;

    loop {
        let ms_ahora = chrono::offset::Local::now().timestamp_millis();
        let diferencial_tiempo = ms_ahora - ms_ultima_iteracion;

        let movimiento = rapidez * diferencial_tiempo as f64;

        posicion = posicion.mover_en_direccion(movimiento, direccion);

        let distancia_a_destino = posicion.distancia(&destino);

        bateria -= descarga_bateria;


        if bateria < 10. {
            destino = Coordenadas::from_lat_lon(-34.6040, -58.3619);
        }


        if distancia_a_destino < 1. {
            // Llegamos al destino yey!

            // si destino es estación de carga, cargar bateria

            rapidez = 0.;
        }

        if distancia_a_destino > 1. {
            // Ajustar direccion
            let diff_lat = destino.lat - posicion.lat;
            let diff_lon = destino.lon - posicion.lon;
            let hipotenusa = (diff_lat.powi(2) + diff_lon.powi(2)).sqrt();
            let direccion = (diff_lat / hipotenusa).acos().to_degrees();
        }
        

        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

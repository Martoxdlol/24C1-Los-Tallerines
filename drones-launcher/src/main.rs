use rand::Rng;
use std::{fs::read_dir, thread}; // 0.8.5

use drone::{comunicacion::Comunicacion, sistema::Sistema};
use lib::{configuracion::Configuracion, dron::Dron};

fn main() {
    let config = Configuracion::desde_argv().expect("Error al cargar configuración");

    let carpet_drones = config
        .obtener::<String>("drones")
        .unwrap_or("drones".to_string());

    let drones_config_rutas = read_dir(carpet_drones).expect("Error al leer directorio de drones");

    let mut drones = Vec::<Dron>::new();

    for ruta in drones_config_rutas {
        let ruta = ruta.expect("Error al leer ruta de dron").path();

        let config_dron =
            Configuracion::leer(ruta.to_str().expect("No se puede cargar la ruta del dron"))
                .expect("Error al leer configuración de dron");

        drones.push(
            Dron::crear(&config_dron)
                .expect(&format!("Configuración de dron incompleta: {:?}", ruta)),
        );
    }

    for mut dron in drones {
        let comunicacion = Comunicacion::new(&config);

        thread::spawn(move || {
            let bateria = rand::thread_rng().gen_range(20..100);

            dron.bateria_actual = bateria as f32 as f64;

            let mut sistema = Sistema::new(dron, comunicacion);

            let num = rand::thread_rng().gen_range(0..3000);

            thread::sleep(std::time::Duration::from_millis(num));

            sistema.iniciar();
        });
    }

    loop {
        thread::park();
    }
}

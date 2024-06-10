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

    let direccion = config
        .obtener("direccion")
        .unwrap_or("127.0.0.1".to_string());
    let puerto = config.obtener("puerto").unwrap_or(4222);
    let user: Option<String> = config.obtener("user");
    let pass: Option<String> = config.obtener("pass");

    for ruta in drones_config_rutas {
        let ruta = ruta
            .expect("Error al leer ruta de dron")
            .path()
            .to_str()
            .expect("No se puede cargar la ruta del dron")
            .to_string();

        let mut config_dron =
            Configuracion::leer(&ruta).expect("Error al leer configuración de dron");

        let mut nombre = ruta.split(&['/', '\\'][..]).last().unwrap();
        nombre = nombre.split('.').next().unwrap();

        if let Ok(id) = nombre.parse::<u64>() {
            config_dron.setear("id", id);
        }

        if config_dron.obtener::<String>("direccion").is_none() {
            config_dron.setear("direccion", direccion.clone());
        }

        if config_dron.obtener::<u16>("puerto").is_none() {
            config_dron.setear("puerto", puerto);
        }

        if user.is_some() && config_dron.obtener::<String>("user").is_none() {
            config_dron.setear("user", user.clone().unwrap());
        }

        if pass.is_some() && config_dron.obtener::<String>("pass").is_none() {
            config_dron.setear("pass", pass.clone().unwrap());
        }

        println!("{:?}", config_dron);

        drones.push(
            Dron::crear(&config_dron)
                .unwrap_or_else(|| panic!("Configuración de dron incompleta: {:?}", nombre)),
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

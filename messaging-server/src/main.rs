use std::{num::NonZero, thread::available_parallelism};

use lib::configuracion::Configuracion;
use messaging_server::servidor::Servidor;

fn main() {
    if let Ok(config) = Configuracion::desde_argv() {
        let mut servidor = Servidor::desde_configuracion(config);

        if let Some(ruta_archivo_cuentas) = servidor.configuracion.obtener::<String>("cuentas") {
            if let Err(e) = servidor.cargar_cuentas(ruta_archivo_cuentas) {
                eprintln!("Error al cargar las cuentas: {}", e);
                return;
            }

            println!("Cuentas cargadas correctamente");
        }

        let cpus = available_parallelism()
            .unwrap_or(NonZero::new(4).expect("No se pudo obtener el número de hilos"));

        println!(
            "Iniciando servidor con {} hilos",
            servidor
                .configuracion
                .obtener::<usize>("hilos")
                .unwrap_or(cpus.into())
        );

        servidor.inicio();
    } else {
        eprintln!("Error al cargar la configuración")
    }
}

use std::{io, net::TcpListener};

use server::conexion::Conexion;

mod server;
fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    listener
        .set_nonblocking(true) // Hace que el listener no bloquee el hilo principal
        .expect("No se pudo poner el listener en modo no bloqueante");

    let mut conexiones: Vec<Conexion> = Vec::new();

    loop {
        // Acepto conexiones nuevas
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(true).unwrap();
                let conexion = Conexion::new(stream); // Si escucho algo, genero una nueva conexion
                conexiones.push(conexion); // Agrego la conexion al vector
                
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No hay conexiones nuevas
            }
            Err(e) => {
                panic!("Error: {}", e);
            }
        }

        // Todas las publicaciones que se recibieron en esta iteración
        let mut todas_las_publicaciones = Vec::new();

        // Por cada conexión,
        for conexion in &mut conexiones {
            // Ejecutar el tick (cada tick recibe, procesa y envía mensajes pendientes)
            conexion.tick();

            // Extraer las publicaciones salientes que se pueden haber generado en el tick
            let publicaciones_salientes = conexion.extraer_publicaciones_salientes();

            // Agregar las publicaciones salientes al vector de publicaciones
            todas_las_publicaciones.extend(publicaciones_salientes);
        }

        // Por cada conexión y por cada cliente, enviarle todas las publicaciones
        // Cada conexión decide si debe enviar el mensaje o ignorarlo según el tópico
        for conexion in &mut conexiones {
            for publicacion in &todas_las_publicaciones {
                conexion.recibir_mensaje(publicacion.clone());
            }
        }
    }
}

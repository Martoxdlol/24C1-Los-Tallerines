pub mod comando;
pub mod respuesta;
use std::{
    io,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use self::{comando::Comando, respuesta::Respuesta};

pub fn interfaz() -> (Sender<Respuesta>, Receiver<Comando>) {
    let (enviar_comando, recibir_comandos) = mpsc::channel::<Comando>();
    let (enviar_respuesta, recibir_respuestas) = mpsc::channel::<Respuesta>();

    thread::spawn(move || loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let comando = interpretar_comando(&input.trim());

        if let Some(comando) = comando {
            if let Err(_e) = enviar_comando.send(comando) {
                break;
            }

            if let Ok(r) = recibir_respuestas.recv() {
                println!("{}", r.como_string());
            }
        } else {
            println!("Comando inválido");
        }
    });

    (enviar_respuesta, recibir_comandos)
}

fn interpretar_comando(input: &str) -> Option<Comando> {
    let mut palabras = input.split_whitespace();
    match palabras.next() {
        Some("conectar") => {
            let id = palabras.next()?.parse().ok()?;
            let lat = palabras.next()?.parse().ok()?;
            let lon = palabras.next()?.parse().ok()?;
            let rango = palabras.next()?.parse().ok().unwrap_or(50.1);
            Some(Comando::Conectar(id, lat, lon, rango))
        }
        Some("desconectar") => {
            let id = palabras.next()?.parse().ok()?;
            Some(Comando::Desconectar(id))
        }
        Some("listar") => Some(Comando::ListarCamaras),
        Some("modificar ubicacion") => {
            let id = palabras.next()?.parse().ok()?;
            let lat = palabras.next()?.parse().ok()?;
            let lon = palabras.next()?.parse().ok()?;
            Some(Comando::ModificarUbicacion(id, lat, lon))
        }
        Some("modificar rango") => {
            let id = palabras.next()?.parse().ok()?;
            let rango = palabras.next()?.parse().ok()?;
            Some(Comando::ModifciarRango(id, rango))
    }
        Some("ayuda") => Some(Comando::Ayuda),
        _ => None,
    }
}

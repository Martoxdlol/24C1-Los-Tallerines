use std::{sync::mpsc::Receiver, thread};

use super::registro::Registro;

pub fn hilo_registrador(rx: Receiver<Registro>) {
    // mientras se reciba un registro, se verifica el "nivel" del registro, y se imprime
    thread::spawn(move || {
        while let Ok(registro) = rx.recv() {
            match registro.nivel {
                super::registro::NivelRegistro::Advertencia => {
                    eprintln!("{}", registro.to_string())
                }
                _ => {
                    println!("{}", registro.to_string())
                }
            }
        }
    });
}

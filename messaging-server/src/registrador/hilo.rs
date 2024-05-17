use std::{sync::mpsc::Receiver, thread};

use super::registro::Registro;

pub fn hilo_registrador(rx: Receiver<Registro>) {
    thread::spawn(move || {
        while let Ok(registro) = rx.recv() {
            match registro.nivel {
                super::registro::NivelRegistro::Advertencia => {
                    eprintln!("{}", registro)
                }
                _ => {
                    println!("{}", registro)
                }
            }
        }
    });
}

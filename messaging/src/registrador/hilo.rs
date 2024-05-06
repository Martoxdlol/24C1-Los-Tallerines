use std::{sync::mpsc::Receiver, thread};

use super::registro::Registro;

pub fn hilo_registrador(rx: Receiver<Registro>) {
    thread::spawn(move || loop {
        match rx.recv() {
            Ok(registro) => println!("{}", registro.to_string()),
            Err(_) => break,
        }
    });
}

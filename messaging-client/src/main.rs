/*
use std::io::prelude::*;
use std::net::TcpStream;
mod client;

fn main() {
    let mut stream = TcpStream::connect("localhost:3001").unwrap();

    let mut linea: Vec<u8> = Vec::new();

    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        // Cantidad de bytes que acabo de leer
        let n = stream.read(&mut buf).unwrap();

        let salto_de_linea = encontrar_salto_de_linea(&buf[..n]);

        // i posición salto de linea
        if let Some(i) = salto_de_linea {
            linea.extend_from_slice(&buf[..i]);
            // Completé la linea
            let linea_como_str = String::from_utf8_lossy(&linea).to_string();
            line_completa(&linea_como_str, &mut stream);
            linea.clear();
        } else {
            linea.extend_from_slice(&buf[..n]);
            // Linea está incompleta
        }

        // print!("{}", String::from_utf8_lossy(&buf[..n]));
    }
}

fn line_completa(linea: &str, stream: &mut TcpStream) {
    // Parsearla
    // y hacer algo...
    if linea.starts_with("INFO") {
        stream.write_all(b"CONNECT {}\r\n").unwrap();
    }

    if linea.starts_with("PING") {
        stream.write_all(b"PONG\r\n").unwrap();
    }
}

/// TODO: \r\n
fn encontrar_salto_de_linea(buffer: &[u8]) -> Option<usize> {
    for (i, &item) in buffer.iter().enumerate() {
        if item == b'\n' {
            return Some(i);
        }
    }
    None
}
*/

use client::Client;
mod client;
use std::thread;
use std::sync::mpsc;

fn main() -> Result<(), String> {
    let mut client = Client::new();

    // Channel para subscripciones
    let (tx1, rx1) = mpsc::channel();

    // Channel para publicaciones
    let (tx2, rx2) = mpsc::channel();

    // Thread para subscripciones
    let handler_sub = thread::spawn(move || {
        for _ in client {
            tx1.send().unwrap();
        }
    });

    // Thread para publicaciones
    let handler_pub = thread::spawn(move || {
        for _ in client {
            rx2.try_rcv().unwrap();
        }
    });

    if client.connect("localhost:4222").is_ok() {
        println!("Conectado correctamente!");
    }

    for _ in client {}

    handler_sub.join().unwrap();
    handler_pub.join().unwrap();

    Ok(())
}

use core::panic;
//use std::borrow::BorrowMut;
use std::io::prelude::*;
//use std::ops::Deref;
use std::{io, net::TcpStream};

pub struct Client {
    stream: Option<TcpStream>,
    topics: Option<Vec<String>>
}

impl Client {
    pub fn new() -> Self {
        Client{ stream: None, topics: Some(Vec::new())}
    }

    pub fn connect(&mut self, host: String) -> io::Result<()> {
        self.stream = Some(TcpStream::connect(host)?);
        Ok(())
    }

    pub fn get_stream(&self) -> Option<&TcpStream> {
        if let Some(stream_ok) = &self.stream {
            Some(stream_ok)
        } else {
            None
        }
    }

    pub fn get_topics(&self) -> Option<Vec<String>> {
        if let Some(topics) = &self.topics {
            Some(topics.to_vec())
        } else {
            None
        }
    }
}

impl Iterator for Client {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line: Vec<u8> = Vec::new();

        if let Some(mut stream) = &mut self.get_stream() {
            loop {
                let mut buf: [u8; 1024] = [0; 1024];
                // Cantidad de bytes que acabo de leer
                let n: usize = stream.read(&mut buf).unwrap();

                let line_break = find_line_break(&buf[..n]);

                // i posición salto de linea
                if let Some(i) = line_break {
                    line.extend_from_slice(&buf[..i]);
                    // Completé la linea
                    let line_as_str = String::from_utf8_lossy(&line).to_string();

                    line.clear();

                    // Linea está completa
                    if let Some(message) = parse_line(stream, &line_as_str) {
                        return Some(message);
                    }
                } else {
                    line.extend_from_slice(&buf[..n]);
                    // Linea está incompleta
                }

                // print!("{}", String::from_utf8_lossy(&buf[..n]));
            }
        }

        panic!("No hay stream");
    }
}

fn parse_line(mut stream: &TcpStream, line: &str) -> Option<String> {
    // Parsearla
    // y hacer algo...
    let mut respuesta = String::new();
    if line.starts_with("INFO") {
        println!("SERVIDOR: INFO");
        println!(r"CLIENTE: CONNECT {{}}");
        stream.write_all(b"CONNECT {}\r\n").unwrap();
    } else if line.starts_with("PING") {
        println!("SERVIDOR: PING");
        println!(r"CLIENTE: PONG");
        stream.write_all(b"PONG\r\n").unwrap();
    } else if line.starts_with("SUB") {

    } else if line.starts_with("PUB") {

    } else {
        return None;
    }
    Some(line.to_string())
}

/// TODO: \r\n
fn find_line_break(buffer: &[u8]) -> Option<usize> {
    for (i, &item) in buffer.iter().enumerate() {
        if item == b'\n' {
            return Some(i);
        }
    }
    None
}

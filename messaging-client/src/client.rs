use core::panic;
//use std::borrow::BorrowMut;
use std::io::prelude::*;
//use std::ops::Deref;
use std::{io, net::TcpStream};

pub struct Client {
    stream: Option<TcpStream>,
}

impl Client {
    pub fn new() -> Self {
        Client { stream: None }
    }

    pub fn connect(&mut self, host: &str) -> io::Result<()> {
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
}

impl Iterator for Client {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line: Vec<u8> = Vec::new();

        if let Some(mut stream) = &mut self.get_stream() {
            loop {
                let mut buf: [u8; 1024] = [0; 1024];
                // Cantidad de bytes que acabo de leer
                let n = stream.read(&mut buf).unwrap();

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
    if line.starts_with("INFO") {
        stream.write_all(b"CONNECT {}\r\n").unwrap();
    } else if line.starts_with("PING") {
        stream.write_all(b"PONG\r\n").unwrap();
        // stream.write_all(b"SUB asd 1\r\n").unwrap();
        // stream.write_all(b"PUB asd 5\r\n").unwrap();
        // stream.write_all(b"hello\r\n").unwrap();
    } else if line.starts_with("PONG") {
        stream.write_all(b"PING\r\n").unwrap();
    } else if line.starts_with("SUB") || line.starts_with("PUB") {
        stream.write_all(b"PONG\r\n").unwrap();
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

use std::io::prelude::*;
use std::{io, net::TcpStream};

pub enum clientMessages<'a> {
    Connect,
    Pub{topic: &'a str, len_message: usize, message: &'a str},
    Hpub{topic: &'a str},
    Sub{topic: &'a str, subscription_id: Option<u8>},
    Unsub{subscription_id: Option<u8>},
}

pub struct Client<'a> {
    stream: Option<TcpStream>,
    topics: Vec<&'a str>,
}

impl<'a> Client<'a> {
    pub fn new() -> Self {
        Client {
            stream: None,
            topics: Vec::new(),
        }
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

    pub fn get_topics(&self) -> Vec<&str> {
        self.topics.clone()
    }
}

impl Iterator for Client<'_> {
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
    println!("LINE: {}", line);
    
    match line {
        line if line.starts_with("INFO") => send_message_connect(&mut stream, line),
        line if line.starts_with("PING") => {
            send_message_pong(&mut stream, line);
            send_message_sub(&mut stream, line, "asd", Some(1));
            send_message_pub(&mut stream, line, "asd", 5, "hola!");
        }
        _ => return None
    }

    Some(line.to_string())
}

fn send_message_connect(mut stream: &TcpStream, line: &str) {
    stream.write_all(b"CONNECT {}\r\n").unwrap();
    clientMessages::Connect;
}

// QUITAR DESPUÉS
fn send_message_pong(mut stream: &TcpStream, line: &str) {
    stream.write_all(b"PONG\r\n").unwrap();
}

fn send_message_pub(mut stream: &TcpStream, line: &str, topic: &str, len_message: usize, message: &str) {
    // Manejar el error
    if topic.is_empty() || len_message == 0 || message.is_empty() {
        return;
    }
    // stream.write_all(b"PUB {topic} {len_message}\r\n").unwrap();
    // stream.write_all(b"{message}\r\n").unwrap();
    stream.write_all(b"PUB asd 5\r\n").unwrap();
    stream.write_all(b"hola!\r\n").unwrap();
    clientMessages::Pub { topic, len_message, message };
}

fn send_message_sub(mut stream: &TcpStream, line: &str, topic: &str, subscription_id: Option<u8>) {
    // Manejar el error
    if topic.is_empty() || subscription_id.is_none() {
        return;
    }
    // stream.write_all(b"SUB {topic} {subscription_id} \r\n").unwrap();
    stream.write_all(b"SUB asd 5\r\n").unwrap();
    clientMessages::Sub { topic, subscription_id};
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
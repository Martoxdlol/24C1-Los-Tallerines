use std::io::prelude::*;
use std::{io, net::TcpStream};

pub enum ClientMessages<'a> {
    Connect,
    Pub {
        topic: &'a str,
        len_message: Option<usize>,
        message: &'a str,
    },
    Hpub {
        topic: &'a str,
    },
    Sub {
        topic: &'a str,
        subscription_id: Option<u8>,
    },
    Unsub {
        subscription_id: Option<u8>,
    },
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
        line if line.starts_with("INFO") => send_message_connect(stream),
        line if line.starts_with("PING") => {
            send_message_pong(stream);
            send_message_sub(stream, "", None);
            send_message_pub(stream, "", Some(5), "");
            send_message_unsub(stream, None);
        }
        _ => return None,
    }

    Some(line.to_string())
}

fn send_message_connect(mut stream: &TcpStream) {
    stream.write_all(b"CONNECT {}\r\n").unwrap();
    ClientMessages::Connect;
}

// QUITAR DESPUÉS
fn send_message_pong(mut stream: &TcpStream) {
    stream.write_all(b"PONG\r\n").unwrap();
}

fn send_message_pub(
    mut stream: &TcpStream,
    topic: &str,
    len_message: Option<usize>,
    message: &str,
) {
    // Manejar el error
    if topic.is_empty() || len_message.is_none() || len_message == Some(0) || message.is_empty() {
        return;
    }

    let mut pub_len_message = 0;
    if let Some(len_message) = len_message {
        pub_len_message = len_message;
    }

    let message_pub_stream = format!("PUB {} {}\r\n", topic, pub_len_message);
    let message_stream = format!("{}\r\n", message);

    stream
        .write_all(message_pub_stream.as_bytes())
        .unwrap();
    stream
        .write_all(message_stream.as_bytes())
        .unwrap();

    ClientMessages::Pub {
        topic,
        len_message,
        message,
    };
}

fn send_message_sub(mut stream: &TcpStream, topic: &str, subscription_id: Option<u8>) {
    // Manejar el error
    if topic.is_empty() || subscription_id.is_none() {
        return;
    }

    let mut sub_id = 0;
    if let Some(subscription_id) = subscription_id {
        sub_id = subscription_id;
    }

    let message_sub_stream = format!("SUB {} {:?}\r\n", topic, sub_id);

    stream
        .write_all(message_sub_stream.as_bytes())
        .unwrap();

    ClientMessages::Sub {
        topic,
        subscription_id,
    };
}

fn send_message_unsub(mut stream: &TcpStream, subscription_id: Option<u8>) {
    if subscription_id.is_none() {
        return;
    }

    let mut sub_id = 0;
    if let Some(subscription_id) = subscription_id {
        sub_id = subscription_id;
    }

    let message_unsub_stream = format!("UNSUB {:?}\r\n", sub_id);

    stream
        .write_all(message_unsub_stream.as_bytes())
        .unwrap();

    ClientMessages::Unsub { subscription_id };
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

#[cfg(test)]
#[test]
fn test01_assert_correct_connection() {
    assert!()
}

#[test]
fn test02_assert_try_connection_with_invalid_stream() {
    assert!()
}

#[test]
fn test03_assert_send_connect_message() {
    assert!()
}

#[test]
fn test04_assert_send_pub_message() {
    assert!()
}

#[test]
fn test05_assert_send_sub_message() {
    assert!()
}

#[test]
fn test06_assert_send_unsub_message() {
    assert!()
}

#[test]
fn test07_assert_send_hpub_message() {
    assert!()
}

#[test]
fn test08_assert_send_pub_message_without_topic() {
    assert!()
}

#[test]
fn test09_assert_send_pub_message_with_message_length_zero() {
    assert!()
}

#[test]
fn test10_assert_send_pub_message_without_message_length() {
    assert!()
}

#[test]
fn test11_assert_send_pub_message_with_empty_message() {
    assert!()
}

#[test]
fn test12_assert_send_sub_message_without_topic() {
    assert!()
}

#[test]
fn test13_assert_send_sub_message_without_subscription_id() {
    assert!()
}

#[test]
fn test14_assert_send_unsub_message_without_subscription_id() {
    assert!()
}

/*
#[test]
fn test15_ {
    assert!()
}
*/

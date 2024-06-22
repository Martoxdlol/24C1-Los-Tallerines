pub mod mock;
pub mod mock_handler;

use std::{
    io::{Read, Write},
    net::TcpStream,
};

use native_tls::TlsStream;

pub trait Stream: Read + Write + Send {
    // Add methods here if you need more than what's provided by Read and Write
}

impl Stream for TcpStream {}

impl Stream for TlsStream<TcpStream> {}

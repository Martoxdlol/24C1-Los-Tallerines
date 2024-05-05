use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

pub trait Stream: Read + Write + Send {
    // Add methods here if you need more than what's provided by Read and Write
}

impl Stream for TcpStream {}


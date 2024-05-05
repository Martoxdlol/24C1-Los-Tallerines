use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

pub trait Stream: Read + Write + Send {
    // Add methods here if you need more than what's provided by Read and Write
}

impl Stream for TcpStream {}

struct MockStream {}

impl Stream for MockStream {}

impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // Mock implementation here
        todo!()
    }
}

impl Write for MockStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Mock implementation here
        todo!()
    }

    fn flush(&mut self) -> io::Result<()> {
        // Mock implementation here
        todo!()
    }
}

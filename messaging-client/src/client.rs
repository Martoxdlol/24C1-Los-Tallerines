use std::{io, net::TcpStream};

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn conectar(address: &str) -> io::Result<Client> {
        let stream = TcpStream::connect(address)?;

        Ok(Client { stream })
    }

    pub fn publicar(
        &mut self,
        subject: &str,
        body: &[u8],
        reply_to: Option<&str>,
    ) -> io::Result<()> {
        todo!();
    }

    pub fn publish_con_headers(
        &mut self,
        subject: &str,
        body: &[u8],
        headers: &[u8],
        reply_to: Option<&str>,
    ) -> io::Result<()> {
        todo!();
    }

    pub fn subscribe(&mut self, subject: &str, queue_group: Option<&str>) {
        todo!();
    }
}

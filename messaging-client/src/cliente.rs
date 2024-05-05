use std::{
    io,
    net::TcpStream,
    sync::mpsc::{SendError, Sender},
    thread::{self, JoinHandle},
};

use crate::{
    hilo_cliente::HiloCliente, instruccion::Instruccion, publicacion::Publicacion,
    subscripcion::Subscripcion,
};

pub struct Cliente {
    _hilo_cliente: JoinHandle<()>,
    canal_instrucciones: Sender<Instruccion>,
    id: usize,
}

impl Cliente {
    pub fn conectar(address: &str) -> io::Result<Cliente> {
        let stream = TcpStream::connect(address)?;
        stream.set_nonblocking(true)?;

        let (tx, rx) = std::sync::mpsc::channel();

        let _hilo_cliente = thread::spawn(move || {
            let mut hilo_cliente = HiloCliente::new(stream, rx);
            hilo_cliente.ejecutar().unwrap();
        });

        Ok(Cliente {
            _hilo_cliente,
            canal_instrucciones: tx,
            id: 0,
        })
    }

    pub fn publicar(
        &mut self,
        subject: &str,
        body: &[u8],
        reply_to: Option<&str>,
    ) -> Result<(), SendError<Instruccion>> {
        let publicacion = Publicacion {
            header: None,
            payload: body.to_vec(),
            replay_to: reply_to.map(|s| s.to_owned()),
            subject: subject.to_owned(),
        };

        self.canal_instrucciones
            .send(Instruccion::Publicar(publicacion))?;

        Ok(())
    }

    pub fn publicar_con_header(
        &mut self,
        subject: &str,
        body: &[u8],
        header: &[u8],
        reply_to: Option<&str>,
    ) -> Result<(), SendError<Instruccion>> {
        let publicacion = Publicacion {
            header: Some(header.to_vec()),
            payload: body.to_vec(),
            replay_to: reply_to.map(|s| s.to_owned()),
            subject: subject.to_owned(),
        };

        self.canal_instrucciones
            .send(Instruccion::Publicar(publicacion))?;

        Ok(())
    }

    pub fn suscribirse(
        &mut self,
        subject: &str,
        queue_group: Option<&str>,
    ) -> Result<Subscripcion, SendError<Instruccion>> {
        self.id += 1;
        let id = format!("{}", self.id);

        let canal_instrucciones = self.canal_instrucciones.clone();

        let (tx, rx) = std::sync::mpsc::channel::<Publicacion>();

        canal_instrucciones.send(Instruccion::Subscribir {
            topico: subject.to_owned(),
            id_suscripcion: id.to_owned(),
            queue_group: queue_group.map(|s| s.to_owned()),
            canal: tx,
        })?;

        Ok(Subscripcion::new(canal_instrucciones, rx, id))
    }
}

impl Drop for Cliente {
    fn drop(&mut self) {
        let _ = self.canal_instrucciones.send(Instruccion::Desconectar);
    }
}

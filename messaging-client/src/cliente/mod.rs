mod hilo_cliente;
mod instruccion;
mod publicacion;
mod suscripcion;

use std::{
    io::{self, Error, ErrorKind},
    net::TcpStream,
    sync::mpsc::{channel, Sender},
    thread::{self, JoinHandle},
    time::Duration
};

use self::{
    hilo_cliente::HiloCliente, instruccion::Instruccion, publicacion::Publicacion,
    suscripcion::Suscripcion,
};

use nuid::NUID;

/// Cliente tiene su hilo donde se gestionan los mensajes, el canal por el cual
/// se envian mensajes al servidor, y el id del cliente
pub struct Cliente {
    _hilo_cliente: JoinHandle<()>,
    canal_instrucciones: Sender<Instruccion>,
    id: usize,
    nuid: NUID
}

impl Cliente {
    pub fn conectar(direccion: &str) -> io::Result<Cliente> {
        let stream = TcpStream::connect(direccion)?;
        stream.set_nonblocking(true)?;

        let (tx, rx) = std::sync::mpsc::channel();

        let hilo_cliente = thread::spawn(move || {
            let mut hilo_cliente = HiloCliente::new(Box::new(stream), rx);
            if let Err(e) = hilo_cliente.ejecutar() {
                eprintln!("Error en hilo cliente: {}", e)
            } else {
                println!("Hilo cliente finalizado")
            }
        });

        Ok(Cliente {
            _hilo_cliente: hilo_cliente,
            canal_instrucciones: tx,
            id: 0,
            nuid: NUID::new()
        })
    }

    pub fn publicar(
        &mut self,
        subject: &str,
        body: &[u8],
        reply_to: Option<&str>,
    ) -> io::Result<()> {
        let publicacion = Publicacion {
            header: None,
            payload: body.to_vec(),
            replay_to: reply_to.map(|s| s.to_owned()),
            subject: subject.to_owned(),
        };

        self.canal_instrucciones
            .send(Instruccion::Publicar(publicacion))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }

    pub fn publicar_con_header(
        &mut self,
        subject: &str,
        body: &[u8],
        header: &[u8],
        reply_to: Option<&str>,
    ) -> io::Result<()> {
        let publicacion = Publicacion {
            header: Some(header.to_vec()),
            payload: body.to_vec(),
            replay_to: reply_to.map(|s| s.to_owned()),
            subject: subject.to_owned(),
        };

        self.canal_instrucciones
            .send(Instruccion::Publicar(publicacion))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }

    pub fn suscribirse(
        &mut self,
        subject: &str,
        queue_group: Option<&str>,
    ) -> io::Result<Suscripcion> {
        //if subject.is_empty() {
        //    return
        //}

        self.id += 1;
        let id: String = format!("{}", self.id);

        let canal_instrucciones = self.canal_instrucciones.clone();

        let (tx, rx) = channel::<Publicacion>();

        canal_instrucciones.send(Instruccion::Suscribir {
            topico: subject.to_owned(),
            id_suscripcion: id.to_owned(),
            queue_group: queue_group.map(|s| s.to_owned()),
            canal: tx,
        })
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(Suscripcion::new(canal_instrucciones, rx, id))
    }

    fn nuevo_inbox(&mut self) -> String {
        format!("_INBOX.{}", self.nuid.next())
    }

    pub fn publicar_request(
        &mut self,
        subject: &str,
        body: &[u8],
        reply_to: Option<&str>,) -> io::Result<()> {
        self.publicar(subject, body, reply_to)
    }

    pub fn request(&mut self, subject: &str, body: &[u8]) -> io::Result<Publicacion> {
        self.request_con_headers_o_timeout(subject, body, None, None)
    }

    pub fn request_con_timeout(&mut self, subject: &str, body: &[u8], timeout: Duration) -> io::Result<Publicacion> {
        self.request_con_headers_o_timeout(subject, body, None, Some(timeout))
    }

    pub fn request_multi(&mut self, subject: &str, body: &[u8]) -> io::Result<Suscripcion> {
        let reply = self.nuevo_inbox();
        let sub = self.suscribirse(&reply, None)?;
        self.publicar(subject, body, Some(reply.as_str()))?;

        Ok(sub)
    }

    fn request_con_headers_o_timeout(
        &mut self,
        subject: &str,
        body: &[u8],
        header: Option<&[u8]>,
        timeout: Option<Duration>,
    ) -> io::Result<Publicacion> {
        // Publicar la request
        let reply = self.nuevo_inbox();
        let mut sub = self.suscribirse(&reply, None)?;
        if let Some(header_ok) = header {
            self.publicar_con_header(subject, body, header_ok, Some(&reply))?;
        } else {
            self.publicar(subject, body, Some(&reply))?;
        }

        // Esperar por una respuesta
        let result: Result<Publicacion, Error>;
        if let Some(timeout_ok) = timeout {
            match sub.leer_con_limite_de_tiempo(timeout_ok) {
                Ok(op_pub) => {
                    if let Some(pub_ok) = op_pub {
                        result = Ok(pub_ok)
                    } else {
                        result = Err(Error::new(ErrorKind::ConnectionAborted, "Timeout superado"))
                    }
                },
                Err(_) => result = Err(Error::new(ErrorKind::ConnectionAborted, "Timeout superado"))
            }
        } else if let Some(res_pub) = sub.next() {
            match res_pub {
                Ok(msg_ok) => result = Ok(msg_ok),
                Err(_) => result = Err(Error::new(ErrorKind::ConnectionAborted, "Timeout superado"))
            }
        } else {
            result = Err(ErrorKind::ConnectionReset.into())
        };

        result
    }

}

impl Drop for Cliente {
    fn drop(&mut self) {
        let _ = self.canal_instrucciones.send(Instruccion::Desconectar);
    }
}

// #[cfg(test)]
// #[test]
// fn test01_assert_cliente_se_conecta_correctamente() {
//     let cliente = Cliente::conectar("localhost:4222");
//     assert!(cliente.is_ok());
// }

// #[test]
// fn test02_assert_cliente_se_suscribe_a_topico_sin_queue_group_correctamente(
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut cliente = Cliente::conectar("localhost:4222")?;
//     let sub = cliente.suscribirse("asd", None);

//     assert!(sub.is_ok());

//     Ok(())
// }

// #[test]
// fn test03_assert_cliente_no_se_suscribe_sin_topico() -> Result<(), Box<dyn std::error::Error>> {
//     let mut cliente = Cliente::conectar("localhost:4222")?;
//     let sub = cliente.suscribirse("", None);

//     assert!(sub.is_err());

//     Ok(())
// }

// #[test]
// fn test04_assert_cliente_se_desuscribe_de_topico_con_id_valido(
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut cliente = Cliente::conectar("localhost:4222")?;
//     let _sub = cliente.suscribirse("abc", None);

//     // Hacer algo

//     Ok(())
// }

// #[test]
// fn test05_assert_cliente_no_se_desuscribe_de_topico_con_id_invalido(
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut cliente = Cliente::conectar("localhost:4222")?;
//     let _sub = cliente.suscribirse("abc", None);

//     // Hacer algo

//     Ok(())
// }

// #[test]
// fn test06_assert_cliente_publica_con_topico_y_mensaje_correctos(
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut cliente = Cliente::conectar("localhost:4222")?;
//     let _sub: Result<Suscripcion, SendError<Instruccion>> = cliente.suscribirse("abc", None);

//     cliente.publicar("asd", b"hola", None)?;
//     // Hacer algo

//     Ok(())
// }

// #[test]
// fn test07_assert_cliente_no_publica_sin_topico() -> Result<(), Box<dyn std::error::Error>> {
//     let mut cliente = Cliente::conectar("localhost:4222")?;
//     let _sub = cliente.suscribirse("abc", None);

//     cliente.publicar("", b"hola", None)?;
//     // Hacer algo

//     Ok(())
// }

// #[test]
// fn test08_assert_cliente_no_publica_sin_mensaje() -> Result<(), Box<dyn std::error::Error>> {
//     let mut cliente = Cliente::conectar("localhost:4222")?;
//     let _sub = cliente.suscribirse("abc", None);

//     cliente.publicar("asd", b"", None)?;
//     // Hacer algo

//     Ok(())
// }

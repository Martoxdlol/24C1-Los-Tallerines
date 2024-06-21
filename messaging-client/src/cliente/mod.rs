mod hilo_cliente;
mod instruccion;
pub mod jetstream;
pub mod publicacion;
pub mod suscripcion;

use std::{
    io,
    net::TcpStream,
    sync::mpsc::{channel, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use self::{
    hilo_cliente::HiloCliente, instruccion::Instruccion, publicacion::Publicacion,
    suscripcion::Suscripcion,
};

/// Cliente tiene su hilo donde se gestionan los mensajes, el canal por el cual
/// se envían mensajes al servidor, y el id del cliente
pub struct Cliente {
    _hilo_cliente: JoinHandle<()>,
    canal_instrucciones: Sender<Instruccion>,
    id: usize,
}

impl Cliente {
    pub fn conectar(direccion: &str) -> io::Result<Cliente> {
        Self::conectar_user_pass(direccion, None, None)
    }

    pub fn conectar_user_pass(
        direccion: &str,
        user: Option<String>,
        pass: Option<String>,
    ) -> io::Result<Cliente> {
        let stream = TcpStream::connect(direccion)?;
        stream.set_nonblocking(true)?;

        let (tx, rx) = std::sync::mpsc::channel();

        let hilo_cliente = thread::spawn(move || {
            let mut hilo_cliente = HiloCliente::new(Box::new(stream), rx);
            hilo_cliente.user = user;
            hilo_cliente.pass = pass;
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
        })
    }

    pub fn publicar(&self, subject: &str, body: &[u8], reply_to: Option<&str>) -> io::Result<()> {
        let publicacion = Publicacion {
            header: None,
            payload: body.to_vec(),
            reply_to: reply_to.map(|s| s.to_owned()),
            subject: subject.to_owned(),
        };

        if let Err(e) = self
            .canal_instrucciones
            .send(Instruccion::Publicar(publicacion))
        {
            return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
        }

        Ok(())
    }

    pub fn publicar_con_header(
        &self,
        subject: &str,
        body: &[u8],
        header: &[u8],
        reply_to: Option<&str>,
    ) -> io::Result<()> {
        let publicacion = Publicacion {
            header: Some(header.to_vec()),
            payload: body.to_vec(),
            reply_to: reply_to.map(|s| s.to_owned()),
            subject: subject.to_owned(),
        };

        if let Err(e) = self
            .canal_instrucciones
            .send(Instruccion::Publicar(publicacion))
        {
            return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
        }

        Ok(())
    }

    pub fn nuevo_inbox(&self) -> String {
        format!("_INBOX.{}", nuid::next())
    }

    pub fn peticion(&mut self, subject: &str, body: &[u8]) -> io::Result<Publicacion> {
        if let Some(publicacion) =
            self.peticion_tiempo_limite_o_header(subject, body, None, None)?
        {
            Ok(publicacion)
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "No se recibió respuesta".to_string(),
            ))
        }
    }

    pub fn peticion_tiempo_limite(
        &mut self,
        subject: &str,
        body: &[u8],
        tiempo_limite: Duration,
    ) -> io::Result<Option<Publicacion>> {
        self.peticion_tiempo_limite_o_header(subject, body, None, Some(tiempo_limite))
    }

    pub fn peticion_con_header(
        &mut self,
        subject: &str,
        header: &[u8],
        body: &[u8],
    ) -> io::Result<Publicacion> {
        if let Some(publicacion) =
            self.peticion_tiempo_limite_o_header(subject, body, Some(header), None)?
        {
            Ok(publicacion)
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "No se recibió respuesta".to_string(),
            ))
        }
    }

    pub fn peticion_tiempo_limite_con_header(
        &mut self,
        subject: &str,
        body: &[u8],
        header: &[u8],
        tiempo_limite: Duration,
    ) -> io::Result<Option<Publicacion>> {
        self.peticion_tiempo_limite_o_header(subject, body, Some(header), Some(tiempo_limite))
    }

    fn peticion_tiempo_limite_o_header(
        &mut self,
        subject: &str,
        body: &[u8],
        header: Option<&[u8]>,
        tiempo_limite: Option<Duration>,
    ) -> io::Result<Option<Publicacion>> {
        let inbox = self.nuevo_inbox();
        let suscripcion = self.suscribirse(&inbox, None)?;

        if let Some(header) = header {
            self.publicar_con_header(subject, body, header, Some(&inbox))?;
        } else {
            self.publicar(subject, body, Some(&inbox))?;
        }

        if let Some(tiempo_limite) = tiempo_limite {
            let publicacion = suscripcion.leer_con_limite_de_tiempo(tiempo_limite)?;
            Ok(publicacion)
        } else {
            let publicacion = suscripcion.leer()?;
            Ok(Some(publicacion))
        }
    }

    pub fn suscribirse(
        &mut self,
        subject: &str,
        queue_group: Option<&str>,
    ) -> io::Result<Suscripcion> {
        self.id += 1;
        let id: String = format!("{}", self.id);

        let canal_instrucciones = self.canal_instrucciones.clone();

        let (tx, rx) = channel::<Publicacion>();

        if let Err(e) = canal_instrucciones.send(Instruccion::Suscribir {
            topico: subject.to_owned(),
            id_suscripcion: id.to_owned(),
            queue_group: queue_group.map(|s| s.to_owned()),
            canal: tx,
        }) {
            return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
        }

        Ok(Suscripcion::new(canal_instrucciones, rx, id))
    }
}

impl Drop for Cliente {
    fn drop(&mut self) {
        let _ = self.canal_instrucciones.send(Instruccion::Desconectar);
    }
}

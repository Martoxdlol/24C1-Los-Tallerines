use crate::servidor::Servidor;
use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
};

#[cfg(test)]
#[test]
fn info_connect() {
    use std::io::{BufWriter, Write};

    let stream = config_stream();
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    let linea = proxima_linea(&mut reader);
    assert!(linea.starts_with("INFO"));
    writer.write_all(b"CONNECT {}\r\n").unwrap();
    let linea = proxima_linea(&mut reader);
    println!("{}", linea);
    assert!(linea.starts_with("-OK"));
}

// #[test]
// fn sub_todos() {
//     use std::io::Write;
//     let stream = TcpStream::connect("127.0.0.1:3000").unwrap();
//     let (mut reader, mut writer) = config_stream(&stream);

//     proxima_linea(&mut reader);
//     writer.write_all(b"CONNECT {}").unwrap();
//     proxima_linea(&mut reader);

//     writer.write_all(b"SUB > 1").unwrap();
//     assert!(proxima_linea(&mut reader).starts_with("-OK"));
//     writer.write_all(b"PUB test 4\r\nhola\r\n").unwrap();
//     assert!(proxima_linea(&mut reader).starts_with("-OK"));
//     assert!(proxima_linea(&mut reader).starts_with("MSG test 4"));
// }

#[allow(dead_code)]
fn config_stream() -> TcpStream {
    Servidor::iniciar(Servidor::procesos(4));

    std::thread::sleep(std::time::Duration::from_secs(1));

    let stream = TcpStream::connect("127.0.0.1:3000").unwrap();

    return stream;
}

#[allow(dead_code)]
fn proxima_linea(reader: &mut BufReader<&TcpStream>) -> String {
    let mut buffer = String::new();
    reader.read_line(&mut buffer).unwrap();
    buffer
}

#[allow(dead_code)]
fn proxima_lina_no_ok(reader: &mut BufReader<&TcpStream>) -> String {
    loop {
        let linea = proxima_linea(reader);
        if !linea.starts_with("-OK") {
            return linea;
        }
    }
}

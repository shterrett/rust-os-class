use std::net::TcpStream;
use std::io::Read;
use std::io;
use std::str;
use path::{ Path, path };

pub struct Request {
    pub stream: TcpStream,
    pub path: io::Result<Path>
}

pub fn build_request(mut stream: TcpStream) -> Request {
    let path = read_request(&mut stream);
    Request {
        stream: stream,
        path: path
    }
}

fn read_request(stream: &mut TcpStream) -> io::Result<Path> {
    let mut buf = [0 ;500];
    stream.read(&mut buf).unwrap();
    match str::from_utf8(&buf) {
        Err(error) => {
            println!("Received request error:\n{}", error);
            Err(io::Error::new(io::ErrorKind::Other, error))
        }
        Ok(body) => {
            let req_path = path(body);
            println!("Recieved request body:\n{}", body);
            println!("Requested Path: {}\n", req_path);
            Ok(req_path)
        }
    }
}

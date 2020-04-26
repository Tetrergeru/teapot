
use std::io::{Write, BufReader, Read, BufRead};
use http::{Request, Response};
use std::net::{TcpListener, TcpStream};
use std::thread;

pub trait RequestHandler {
    fn handle(&self, request: Request<Vec<u8>>) -> Response<Vec<u8>>;
}

pub fn listen(adress: &str, routing: Vec<(String, &dyn RequestHandler)>) {
    let listener = TcpListener::bind(adress).unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(move || {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut responder = stream.try_clone().unwrap();
            let _ = super::request_parser::parse(&mut reader);

            let _ = responder.write_all(b"HTTP/1.0 200 OK\n\r     Content-Type:      text/html\n\r\n\r<body>Oh fuck, oh shit</body>");
        });
    }
}
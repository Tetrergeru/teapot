extern crate http;
extern crate regex; 


#[cfg(test)]
mod tests;

mod requester;
mod responser;

use std::io::BufReader;
use std::io::Write;
use http::{Request, Response};
use std::net::{TcpListener};
use std::thread;
use regex::Regex;

pub trait RequestHandler : Sync {
    fn handle(&self, request: Request<Vec<u8>>) -> Response<Vec<u8>>;
}


//pub type Hndle = dyn Fn (Request<Vec<u8>>) -> Response<Vec<u8>>;

pub fn listen<'a>(adress: &'a str, routing: &'static dyn RequestHandler) {
    let listener = TcpListener::bind(adress).unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        thread::spawn(move || {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut responder = stream.try_clone().unwrap();
            let request = requester::parse(&mut reader);
            
            let response = routing.handle(request);

            let response = responser::to_bytes(response);
            //println!("{}", String::from_utf8(response.clone()).unwrap());
            let _ = responder.write_all(&response[0..response.len()]);
        });
    }
}
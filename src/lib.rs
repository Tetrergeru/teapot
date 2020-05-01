extern crate http;
extern crate regex; 


#[cfg(test)]
mod tests;

mod requester;
mod responser;

use std::net::TcpListener;
use std::io::BufReader;
use std::io::Write;

use std::str::FromStr;

use http::{Request, Response};

pub fn route<S>(
        request: Request<Vec<u8>>,
        storage: &mut S,
        routing: &[(regex::Regex, &dyn Fn(Request<Vec<u8>>, &mut S) -> Response<Vec<u8>>)]
    ) -> Option<Response<Vec<u8>>>
    {
    let (head, body) = request.into_parts();
    let path: &str = head.uri.path();
    for (re, handler) in routing.iter() {
        if (*re).is_match(path) {
            return Some(handler(http::Request::from_parts(head, body), storage));
        }
    }
    None
}

pub fn listen<'a, T, S>(adress: &'a str, routing: & T, storage: &mut S) 
    where
        T : Fn(Request<Vec<u8>>, &mut S) -> Response<Vec<u8>>
{
    let listener = TcpListener::bind(adress).unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut responder = stream.try_clone().unwrap();
        let request = requester::parse(&mut reader);
        
        let response = routing(request, storage);

        let (mut head, body) = response.into_parts();
        head.headers.append(http::header::HeaderName::from_str("Content-Length").unwrap(), http::HeaderValue::from_str(&format!("{}", body.len() + 1)).unwrap());
        let response = Response::from_parts(head, body);

        let response = responser::to_bytes(response);
        let _ = responder.write_all(&response[0..response.len()]);
    }
}

#[macro_export]
macro_rules! routing_table {
    ( $name:ident; $count:expr; $storage_type:ty; $($path:expr => $func:ident);* ) => {
        let $name: &[(regex::Regex, & dyn Fn(http::Request<Vec<u8>>, &mut $storage_type) -> http::Response<Vec<u8>>)] = &[
            $(
                (regex::Regex::new($path).unwrap(), & $func),
            )*
        ];
    }
}

#[macro_export]
macro_rules! router {
    (routing => $name:expr, _ => $not_found:expr) => {
        &move |request, storage| {
            match teapot::route::<String>(request, storage, $name) {
                Some(response) => response,
                None => $not_found
            }
        }
    }
}
extern crate http;
extern crate regex; 
extern crate log;
extern crate percent_encoding;

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

const RESP_NOT_FOUND: &str = "<!DOCTYPE HTML PUBLIC>
<html>
	<head>
	</head>
    <body>
        404, Not Found
	</body>
</html>";

pub fn default_404() -> http::Response<Vec<u8>> {
    println!("{}", 404);
    let response = http::response::Builder::new()
        .status("404")
        .header("Content-Type", "text/html");
    response.body(RESP_NOT_FOUND.as_bytes().iter().cloned().collect()).unwrap()
}

pub fn content_type(file_ext: &str) -> String {
    match file_ext {
        "html" => "text/html",
        "css" => "text/css",
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "ico" => "image/icon",
        _ => "text/plain"
    }.to_string()
}

pub fn from_file(path: &str) -> http::Response<Vec<u8>> {
    println!("path = {}", path);
    let extension = std::path::Path::new(path).extension().unwrap().to_str().unwrap();
    let response = http::response::Builder::new()
        .status("200")
        .header("Content-Type", content_type(extension));
    let data = match std::fs::read(&path).ok() {
        None => return default_404(),
        Some(data) => data
    };
    response.body(data).unwrap()
}

pub fn get_from_file<T>(request: http::Request<Vec<u8>>, _storage: &mut T) -> http::Response<Vec<u8>> {
    let parts = request.into_parts();
    let path = parts.0.uri.path();
    println!("path = {}", path);
    from_file(&path[1..])
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
        let request = match requester::parse(&mut reader) {
            None => {
                println!("An error has occured");
                continue;
            }
            Some(request) => request
        };
        
        let response = routing(request, storage);

        let (mut head, body) = response.into_parts();
        head.headers.append(http::header::HeaderName::from_str("Content-Length").unwrap(), http::HeaderValue::from_str(&format!("{}", body.len())).unwrap());
        let response = Response::from_parts(head, body);

        let response = responser::to_bytes(response);
        let _ = responder.write_all(&response[0..]);
    }
}

pub fn parse_args(args: &String) -> std::collections::HashMap<String, String> {
    let mut result = std::collections::HashMap::new();
    for sp in args.split("&") {
        let sp = sp.to_string();
        let sp: Vec<&str> = sp.split("=").collect();
        result.insert(sp[0].to_string(), sp[1].to_string());
    }

    result
}

pub fn parse_cookies(headers: &http::HeaderMap) -> std::collections::HashMap<String, String> {
    let mut result = std::collections::HashMap::new();
    for h in headers.get_all("cookie") {
        for sp in h.to_str().unwrap().to_string().split("; ") {
            let sp = sp.to_string();
            let sp: Vec<&str> = sp.split("=").collect();
            result.insert(sp[0].to_string(), sp[1].to_string());
        }
    } 
    result
}

pub fn url_decode(text: &String) -> String {
    let text = text.replace("+", " ");
    percent_encoding::percent_decode_str(&text).decode_utf8_lossy().to_string()
}

#[macro_export]
macro_rules! routing_table {
    ( $name:ident; $count:expr; $storage_type:ty; $($path:expr => $func:expr);* ) => {
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
            match teapot::route(request, storage, $name) {
                Some(response) => response,
                None => $not_found
            }
        }
    }
}
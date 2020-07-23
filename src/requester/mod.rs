extern crate http;
extern crate regex; 
extern crate log;

use http::{Request, header::HeaderName, Method, Uri, Version};
use std::io::BufRead;
use std::collections::HashMap;
use std::str::FromStr;
use regex::Regex;

pub fn parse(mut reader: &mut dyn BufRead) -> Option<Request<Vec<u8>>> {
    let mut buf_iter = BufIter::new(&mut reader);

    let (first_line, headers) = parse_head(&mut buf_iter);
    log::debug!("{}", first_line);
    for h in headers.clone() {
        log::debug!("{}: {}", h.0, h.1);
    }
    log::debug!("");

    let (method, uri, version) = parse_first_line(&first_line)?;
    
    let mut request = http::request::Builder::new()
        .method(method)
        .uri(uri)
        .version(version);

    let header_map = request.headers_mut()?;

    for h in headers.iter() {
        let name = HeaderName::from_str(&h.0).ok()?;
        let value = h.1.parse().ok()?;
        header_map.append(name, value);
    }

    let body: Vec::<u8> = match headers.get(&"Content-Length".to_string()) {
        None => Vec::new(),
        Some(length) => buf_iter.leftovers(length.parse::<usize>().ok()?)
    };
    request.body(body).ok()
}

struct BufIter<'a> {
    stream: Box<&'a mut dyn BufRead>,
    payload: [u8; 512],
    payload_size: usize,
    current_position: usize,
}

impl BufIter<'_> {
    fn new<'a>(stream: &'a mut dyn BufRead) -> BufIter {
        BufIter {
            stream: Box::new(stream),
            payload: [0; 512],
            payload_size: 1,
            current_position:0,
        }
    }

    fn leftovers(&mut self, size: usize) -> Vec<u8> {
        let mut size = size;
        let mut result = Vec::new();
        if size == 0 {
            return result;
        }
        for byte in self {
            result.push(byte);
            size -= 1;
            if size == 0 {
                break;
            }
        };
        result
    }
}

impl<'a> Iterator for BufIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        self.current_position += 1;
        if self.current_position >= self.payload_size {
            self.payload_size = (*self.stream).read(&mut self.payload).unwrap();
            if self.payload_size == 0 {
                return None;
            }
            self.current_position = 0;
            Some(self.payload[0])
        } else {
            if self.current_position == 512 {
                log::debug!("Test {}, {}", self.current_position, self.payload_size);
            }
            Some(self.payload[self.current_position])
        }
    }
}

fn get_line(iter: &mut BufIter) -> Vec<u8> {
    let mut buf = Vec::new();

    let mut prev: u8 = 0x00;
    for curr in iter {
        if prev == 0x0D && curr == 0x0A {
            return buf;
        } else if curr == 0x0D {
            prev = curr;
        } else {
            buf.push(curr);
            prev = curr;
        }
    }
    Vec::new()
}

fn parse_head(mut buf_iter: &mut BufIter) -> (String, HashMap<String, String>) {
    let mut headers = HashMap::<String, String>::new();
    let header_parser = Regex::new("^\\s*([\\w-]+):\\s+(.+)\\s*$").unwrap();

    let mut buf: Vec::<u8>;

    let first_line = String::from_utf8(get_line(&mut buf_iter)).unwrap();
    loop {
        buf = get_line(&mut buf_iter);
        if buf.len() == 0 {
            break
        }

        let s = String::from_utf8(buf).unwrap();

        match header_parser.captures(&s) {
            None => continue, // TODO: Some error maybe
            Some(capture) => headers.insert(capture[1].to_string(), capture[2].to_string())
        };
    }
    (first_line, headers)
}

fn parse_first_line(first_line: &String) -> Option<(Method, Uri, Version)> {
    let parser = Regex::new("^\\s*([A-Z]+)\\s+(\\S+)\\s+HTTP/(\\d\\.\\d)\\s*$").unwrap();
    match parser.captures(&first_line) {
        None => None,
        Some(capture) => Some((
            Method::from_str(&capture[1]).unwrap(),
            Uri::from_str(&capture[2]).unwrap(),
            Version::HTTP_11))
    }
}
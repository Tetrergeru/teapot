extern crate http;
extern crate regex; 

use http::Request;
use std::io::{Write, BufReader, Read, BufRead};
use std::collections::HashMap;
use regex::Regex;

pub fn parse(reader: &mut dyn BufRead) -> Request<Vec<u8>> {
    let (response, headers) = parse_head(reader);
    let content: Vec::<u8> = match headers.get(&"Content-Length".to_string()) {
        None => Vec::new(),
        Some(length) => Vec::new()
    };
    Request::new(content)
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
}

impl<'a> Iterator for BufIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        self.current_position += 1;
        if self.current_position == self.payload_size {
            self.payload_size = (*self.stream).read(&mut self.payload).unwrap();
            if self.payload_size == 0 {
                return None;
            }
            self.current_position = 0;
            Some(self.payload[0])
        } else {
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

fn parse_head(mut reader: &mut dyn BufRead) -> (String, HashMap<String, String>) {
    let mut headers = HashMap::new();
    let header_parser = Regex::new("\\s*([\\w-]+):\\s+(\\S+)\\s*").unwrap();

    let mut buf: Vec::<u8>;
    let mut buf_iter = BufIter::new(&mut reader);

    let response = String::from_utf8(get_line(&mut buf_iter)).unwrap();
    loop {
        buf = get_line(&mut buf_iter);
        if buf.len() == 0 {
            break
        }

        let s = String::from_utf8(buf).unwrap();

        match header_parser.captures(&s) {
            None => continue,
            Some(capture) => headers.insert(capture[1].to_string(), capture[2].to_string())
        };
        println!("{}", s);
    }
    (response, headers)
}


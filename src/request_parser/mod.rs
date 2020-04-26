extern crate http;

use http::Request;
use std::io::{Write, BufReader, Read, BufRead};

pub fn parse(reader: &mut BufRead) -> Request<Vec<u8>> {

    Request::new(Vec::new())
}
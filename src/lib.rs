extern crate http;

#[cfg(test)]
mod tests;
mod request_parser;

pub mod teapot {
    use http::{Request, Response};

    trait RequestHandler {
        fn handle(&self, request: Request<Vec<u8>>) -> Response<Vec<u8>>;
    }

    pub fn listen(adress: &str, routing: Vec<(String, &dyn RequestHandler)>) {

    }
}
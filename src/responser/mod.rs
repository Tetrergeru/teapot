use http::Response;

fn append<T: Iterator<Item = u8>>(vec: &mut Vec<u8>, iter: T) {
    for i in iter {
        vec.push(i);
    }
}

fn version_to_str(version: http::Version) -> String {
    match version {
        http::Version::HTTP_09 => "HTTP/0.9",
        http::Version::HTTP_10 => "HTTP/1.0",
        http::Version::HTTP_11 => "HTTP/1.1",
        http::Version::HTTP_2 => "HTTP/2.0",
        http::Version::HTTP_3 => "HTTP/3.0",
        _ => panic!("Unknown version")
    }.to_string()
}

pub fn to_bytes(response: Response<Vec<u8>>) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();

    let (head, body) = response.into_parts();
    append(&mut bytes,
        format!("{} {} OK\n\r", version_to_str(head.version), head.status.as_str())
            .as_bytes() .iter() .cloned());
    for header in head.headers {
        println!("!");
        let h = header.0.unwrap().to_string();
        println!("{}: {}", h, &header.1.to_str().unwrap());
        append(&mut bytes, format!("{}: {}\n\r", h, header.1.to_str().unwrap()).as_bytes().iter().cloned());
    }
    append(&mut bytes, b"\n\r".iter().cloned());
    append(&mut bytes, body.iter().cloned());

    bytes
}
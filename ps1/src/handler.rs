use std::io::{ Read, Write };
use std::fs::File;
use path::Path;
use http;

pub fn handle_request<T: Write>(path: Path, visitor_count: u16, stream: &mut T) -> http::Status {
    let response_status =
        router(path, visitor_count)
            .and_then(|bytes|{
                let header = http::header(&http::Status::Ok);
                stream.write(&header)
                    .and_then(|_| stream.write(&bytes))
                    .map_err(|_| http::Status::Error)
            })
            .map_err(|e| {
                match stream.write(&http::header(&e)) {
                    Ok(_) => e,
                    Err(_) => http::Status::Error
                }
            });

    match response_status {
        Ok(_) => http::Status::Ok,
        Err(e) => e
    }
}

fn router(path: Path, visitor_count: u16) -> Result<Vec<u8>, http::Status> {
    match path {
        Path::Root => root_handler(visitor_count),
        Path::RelPath(path) => file_handler(path)
    }
}

fn root_handler(visitor_count: u16) -> Result<Vec<u8>, http::Status> {
    let response =
        format!("<doctype !html><html><head><title>Hello, Rust!</title>
                <style>body {{ background-color: #111; color: #FFEEAA }}
                        h1 {{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red }}
                        h2 {{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green }}
                </style></head>
                <body>
                <h1>Greetings, Krusty!</h1>
                <h2>Visitor Count: {}<h2>
                </body></html>\r\n",
                visitor_count
            );
    Ok(response.into_bytes())
}

fn file_handler(path: String) -> Result<Vec<u8>, http::Status> {
    let mut bytes: Vec<u8> = Vec::new();
    File::open(path)
        .and_then(|mut f| {
            f.read_to_end(&mut bytes)
        })
        .map(|_| bytes)
        .map_err(|_| http::Status::FileNotFound)
}

#[cfg(test)]
mod test {
    use regex::Regex;
    use path::Path;
    use super::handle_request;

    #[test]
    fn root_handler_writes_html() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Path::Root, 5, &mut output);

        let html = String::from_utf8(output).unwrap();

        let visitor_count = Regex::new(r"Visitor Count: 5").unwrap();
        let hello = Regex::new(r"Greetings, Krusty!").unwrap();

        assert!(visitor_count.is_match(&html));
        assert!(hello.is_match(&html));
    }

    #[test]
    fn file_handler_returns_given_file() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Path::RelPath("test/response.html".to_string()), 5, &mut output);

        let html = String::from_utf8(output).unwrap();

        let response = Regex::new(r"<h1>Test Response</h1>\n").unwrap();

        assert!(response.is_match(&html));
    }
}

use std::io;
use std::io::{ Read, Write, BufReader, BufWriter };
use std::fs::File;
use path::Path;

pub fn handle_request<T: Write>(path: Path, visitor_count: u16, stream: &mut T) {
    stream.write("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n".as_bytes());
    match path {
        Path::Root => root_handler(visitor_count, stream),
        Path::RelPath(path) => file_handler(path, stream)
    }
}

fn root_handler<T: Write>(visitor_count: u16, stream: &mut T) {
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
    stream.write(response.as_bytes()).unwrap();
}

fn file_handler<T: Write>(path: String, stream: &mut T) {
    match File::open(path) {
        Ok(mut f) => {
            if let Err(io_error) = write_file(&mut f, stream) {
                println!("FILE RESULT: {}", io_error);
            }
        },
        Err(_) => {}
    }
}

fn write_file<T: Write>(f: &mut File, stream: &mut T) -> io::Result<bool> {
    let mut tmp: Vec<u8> = Vec::new();
    let mut reader = BufReader::new(f);
    let mut writer = BufWriter::new(stream);
    try!(reader.read_to_end(&mut tmp));
    try!(writer.write_all(&tmp));
    try!(writer.flush());
    Ok(true)
}

#[cfg(test)]
mod test {
    use regex::Regex;
    use super::{ root_handler, file_handler };

    #[test]
    fn root_handler_writes_html() {
        let mut output: Vec<u8> = Vec::new();
        root_handler(5, &mut output);

        let html = String::from_utf8(output).unwrap();

        let visitor_count = Regex::new(r"Visitor Count: 5").unwrap();
        let hello = Regex::new(r"Greetings, Krusty!").unwrap();

        assert!(visitor_count.is_match(&html));
        assert!(hello.is_match(&html));
    }

    #[test]
    fn file_handler_returns_given_file() {
        let mut output: Vec<u8> = Vec::new();
        file_handler("test/response.html".to_string(), &mut output);

        let html = String::from_utf8(output).unwrap();

        assert_eq!(html, "<h1>Test Response</h1>\n".to_string());
    }
}

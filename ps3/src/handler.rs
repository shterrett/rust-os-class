use std::collections::HashSet;
use std::io::{ Read, Write };
use std::fs::File;
use std::path::{ Path, Component };
use path::Path as ReqPath;
use http;

lazy_static!{
    static ref ALLOWED_FILE_TYPES: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert("html");
        s.insert("css");
        s.insert("js");
        s.insert("ico");
        s.insert("png");
        s.insert("gif");
        s.insert("jpg");
        s.insert("jpeg");
        s
    };
}

enum AccessError {
    NotFound,
    OutOfBounds,
    TypeNotAllowed
}

pub fn handle_request<T: Write>(path: ReqPath, visitor_count: u16, stream: &mut T) -> http::Status {
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

fn router(path: ReqPath, visitor_count: u16) -> Result<Vec<u8>, http::Status> {
    match path {
        ReqPath::Root => root_handler(visitor_count),
        ReqPath::RelPath(path) => file_handler(path)
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
    valid_file(Path::new(&path))
        .and_then(|mut f| {
            f.read_to_end(&mut bytes)
                .map_err(|_| AccessError::NotFound)
        })
        .map(|_| bytes)
        .map_err(|e| {
            match e {
                AccessError::NotFound => http::Status::FileNotFound,
                AccessError::OutOfBounds => http::Status::NotAuthorized,
                AccessError::TypeNotAllowed => http::Status::NotAuthorized
            }
        })
}

fn valid_file(path: &Path) -> Result<File, AccessError> {
    if path.is_absolute() {
        Err(AccessError::OutOfBounds)
    } else if path.components().any(|c| c == Component::ParentDir) {
        Err(AccessError::OutOfBounds)
    } else if !valid_file_type(path) {
        Err(AccessError::TypeNotAllowed)
    } else {
        File::open(path)
            .map_err(|_| AccessError::NotFound)
    }
}

fn valid_file_type(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        None => false,
        Some(ext) => ALLOWED_FILE_TYPES.contains(ext)
    }
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

    #[test]
    fn fails_for_nonexistent_file() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Path::RelPath("test/does_not_exist.html".to_string()), 5, &mut output);

        let html = String::from_utf8(output).unwrap();

        let response = Regex::new(r"404 Not Found").unwrap();

        assert!(response.is_match(&html));
    }

    #[test]
    fn fails_for_root_access() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Path::RelPath("/etc/hosts".to_string()), 5, &mut output);

        let html = String::from_utf8(output).unwrap();
        let response = Regex::new(r"401 Not Authorized").unwrap();

        assert!(response.is_match(&html));
    }

    #[test]
    fn fails_for_parent_dir_access() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Path::RelPath("../README.md".to_string()), 6, &mut output);

        let html = String::from_utf8(output).unwrap();
        let response = Regex::new(r"401 Not Authorized").unwrap();

        assert!(response.is_match(&html));
    }

    #[test]
    fn fails_for_embedded_parent_dir_access() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Path::RelPath("test/../../index.html".to_string()), 6, &mut output);

        let html = String::from_utf8(output).unwrap();
        let response = Regex::new(r"401 Not Authorized").unwrap();

        assert!(response.is_match(&html));
    }

    #[test]
    fn fails_for_unallowed_file_type() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Path::RelPath("test/passwords.txt".to_string()), 6, &mut output);

        let html = String::from_utf8(output).unwrap();
        let response = Regex::new(r"401 Not Authorized").unwrap();

        assert!(response.is_match(&html));
    }

    #[test]
    fn not_authorized_supersedes_not_found() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Path::RelPath("test/does_not_exist.txt".to_string()), 6, &mut output);

        let html = String::from_utf8(output).unwrap();
        let response = Regex::new(r"401 Not Authorized").unwrap();

        assert!(response.is_match(&html));
    }
}

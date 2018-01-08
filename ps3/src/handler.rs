use std::collections::HashSet;
use std::io::{ BufReader, Write, copy };
use std::io;
use std::fs::File;
use std::path::{ Path, Component };
use path::Path as ReqPath;
use http::{ header, Status, Payload };
use shell_interpolation::insert_shell_commands;

lazy_static!{
    static ref ALLOWED_FILE_TYPES: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert("shtml");
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

pub fn handle_request<T: Write>(req_path: io::Result<ReqPath>, visitor_count: usize, stream: &mut T) -> Status {
    match req_path {
        Ok(path) => {
            let response_status =
                router(path, visitor_count)
                    .and_then(|mut payload| {
                        let header = header(&Status::Ok);
                        stream.write(&header)
                            .and_then(|_| {
                                match &mut payload {
                                    &mut Payload::Stream(ref mut f) => {
                                        copy(f, stream)
                                    }
                                    &mut Payload::Block(ref s) => {
                                        stream.write(s.as_bytes()).map(|b| b as u64)
                                    }
                                }
                            })
                            .map_err(|_| Status::Error)
                    })
                    .map_err(|e| {
                        match stream.write(&header(&e)) {
                            Ok(_) => e,
                            Err(_) => Status::Error
                        }
                    });

                match response_status {
                    Ok(_) => Status::Ok,
                    Err(e) => e
                }
        }
        Err(_) => Status::Error
    }
}

fn router(path: ReqPath, visitor_count: usize) -> Result<Payload, Status> {
    match path {
        ReqPath::Root => root_handler(visitor_count),
        ReqPath::RelPath(path) => file_handler(path)
    }
}

fn root_handler(visitor_count: usize) -> Result<Payload, Status> {
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
    Ok(Payload::Block(response.to_string()))
}

fn file_handler(path: String) -> Result<Payload, Status> {
    let file_path = Path::new(&path);
    valid_file(&file_path)
        .map(|f| BufReader::new(f) )
        .map_err(|e| {
            match e {
                AccessError::NotFound => Status::FileNotFound,
                AccessError::OutOfBounds => Status::NotAuthorized,
                AccessError::TypeNotAllowed => Status::NotAuthorized
            }
        })
        .and_then(|f| insert_shell_commands(&file_path, f).map_err(|_| Status::Error))
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
    use http::Status;
    use std::io;

    #[test]
    fn root_handler_writes_html() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Ok(Path::Root), 5, &mut output);

        let html = String::from_utf8(output).unwrap();

        let visitor_count = Regex::new(r"Visitor Count: 5").unwrap();
        let hello = Regex::new(r"Greetings, Krusty!").unwrap();

        assert!(visitor_count.is_match(&html));
        assert!(hello.is_match(&html));
    }

    #[test]
    fn file_handler_returns_given_file() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Ok(Path::RelPath("test/response.html".to_string())), 5, &mut output);

        let html = String::from_utf8(output).unwrap();

        let response = Regex::new(r"<h1>Test Response</h1>\n").unwrap();

        assert!(response.is_match(&html));
    }

    #[test]
    fn fails_for_nonexistent_file() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Ok(Path::RelPath("test/does_not_exist.html".to_string())), 5, &mut output);

        let html = String::from_utf8(output).unwrap();

        let response = Regex::new(r"404 Not Found").unwrap();

        assert!(response.is_match(&html));
    }

    #[test]
    fn fails_for_root_access() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Ok(Path::RelPath("/etc/hosts".to_string())), 5, &mut output);

        let html = String::from_utf8(output).unwrap();
        let response = Regex::new(r"401 Not Authorized").unwrap();

        assert!(response.is_match(&html));
    }

    #[test]
    fn fails_for_parent_dir_access() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Ok(Path::RelPath("../README.md".to_string())), 6, &mut output);

        let html = String::from_utf8(output).unwrap();
        let response = Regex::new(r"401 Not Authorized").unwrap();

        assert!(response.is_match(&html));
    }

    #[test]
    fn fails_for_embedded_parent_dir_access() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Ok(Path::RelPath("test/../../index.html".to_string())), 6, &mut output);

        let html = String::from_utf8(output).unwrap();
        let response = Regex::new(r"401 Not Authorized").unwrap();

        assert!(response.is_match(&html));
    }

    #[test]
    fn fails_for_unallowed_file_type() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Ok(Path::RelPath("test/passwords.txt".to_string())), 6, &mut output);

        let html = String::from_utf8(output).unwrap();
        let response = Regex::new(r"401 Not Authorized").unwrap();

        assert!(response.is_match(&html));
    }

    #[test]
    fn not_authorized_supersedes_not_found() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Ok(Path::RelPath("test/does_not_exist.txt".to_string())), 6, &mut output);

        let html = String::from_utf8(output).unwrap();
        let response = Regex::new(r"401 Not Authorized").unwrap();

        assert!(response.is_match(&html));
    }

    #[test]
    fn interpolates_shell_command_in_shtml() {
        let mut output: Vec<u8> = Vec::new();
        handle_request(Ok(Path::RelPath("test/world.shtml".to_string())), 6, &mut output);

        let html = String::from_utf8(output).unwrap();
        let response = Regex::new("<h1>\"Hello World\"\n</h1>").unwrap();

        assert!(response.is_match(&html));
    }

    #[test]
    fn returns_error_if_path_not_parsed() {
        let mut output: Vec<u8> = Vec::new();
        let status = handle_request(Err(io::Error::new(io::ErrorKind::Other, "Whoops")), 6, &mut output);

        assert_eq!(status, Status::Error);
    }
}

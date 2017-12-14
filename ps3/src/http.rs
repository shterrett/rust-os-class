use std::fmt;

pub enum Status {
    Ok,
    FileNotFound,
    Error,
    NotAuthorized
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match self {
            &Status::Ok => write!(f, "200 OK"),
            &Status::FileNotFound => write!(f, "404 Not Found"),
            &Status::Error => write!(f, "500 Internal Server Error"),
            &Status::NotAuthorized => write!(f, "401 Not Authorized")
        }
    }
}

pub fn header(status: &Status) -> Vec<u8> {
    format!("HTTP/1.1 {}\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n", status).into_bytes()
}

#[cfg(test)]
mod test {
    use super::{ Status, header };

    #[test]
    fn formats_status_into_header() {
        assert_eq!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n".to_string().into_bytes(),
                   header(&Status::Ok));
        assert_eq!("HTTP/1.1 404 Not Found\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n".to_string().into_bytes(),
                   header(&Status::FileNotFound));
        assert_eq!("HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n".to_string().into_bytes(),
                   header(&Status::Error));
        assert_eq!("HTTP/1.1 401 Not Authorized\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n".to_string().into_bytes(),
                   header(&Status::NotAuthorized));
    }
}

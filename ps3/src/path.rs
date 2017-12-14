use regex::Regex;
use std::fmt;

lazy_static! {
    static ref PATH_REGEX: Regex = Regex::new(r"GET /(\S*)\s").unwrap();
}

#[derive(Debug, PartialEq, Eq)]
pub enum Path {
    Root,
    RelPath(String)
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Path::Root => write!(f, "/"),
            &Path::RelPath(ref path) => write!(f, "/{}", path)
        }
    }
}

pub fn path(body: &str) -> Path {
    match extract_request_path(body) {
        "" => Path::Root,
        request_path => Path::RelPath(request_path.to_string())
    }
}

fn extract_request_path(body: &str) -> &str {
    PATH_REGEX.captures(body)
              .and_then(|matches| {
                  matches.get(1)
              })
              .map(|m| m.as_str())
              .unwrap_or("")
}

#[cfg(test)]
mod test {
    use super::{Path, path};

    #[test]
    fn returns_root_for_empty_path() {
        let request =
            "GET / HTTP/1.1
            Host: localhost:4414
            Connection: keep-alive
            Pragma: no-cache
            Cache-Control: no-cache
            User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/59.0.3071.115 Safari/537.36
            Accept: image/webp,image/apng,image/*,*/*;q=0.8
            Referer: http://localhost:4414/
            Accept-Encoding: gzip, deflate, br
            Accept-Language: en-US,en;q=0.8";

        let request_path = path(&request);

        assert_eq!(request_path, Path::Root);
    }

    #[test]
    fn returns_relative_path_for_non_empty_path() {
        let request =
            "GET /index.html HTTP/1.1
            Host: localhost:4414
            Connection: keep-alive
            Pragma: no-cache
            Cache-Control: no-cache
            User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/59.0.3071.115 Safari/537.36
            Accept: image/webp,image/apng,image/*,*/*;q=0.8
            Referer: http://localhost:4414/
            Accept-Encoding: gzip, deflate, br
            Accept-Language: en-US,en;q=0.8";

        let request_path = path(&request);

        assert_eq!(request_path, Path::RelPath("index.html".to_string()));
    }
}

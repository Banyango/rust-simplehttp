use nom::{is_alphanumeric};
use std::error::Error;
use std::string::FromUtf8Error;
use std::fmt;
use std::path::PathBuf;

use colored::*;

#[derive(Debug)]
pub struct ParseError {
    description:&'static str,
}

impl ParseError {
    fn new(description:&'static str) -> Self {
        ParseError{
            description:description
        }
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        self.description
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl From<FromUtf8Error> for ParseError {
    fn from(_err: FromUtf8Error) -> Self {
        ParseError::new("Error Parsing URI")
    }
}

impl From<std::path::StripPrefixError> for ParseError {
    fn from(_err: std::path::StripPrefixError) -> Self {
        ParseError::new("Couldn't strip the absolute path")
    }
}

#[derive(PartialEq,Debug,Clone)]
pub enum Method {
    Get,
    Post,
    Head,
    Options,
    Put,
    Delete,
    Trace,
    Connect,
}

impl Method {
    pub fn new(s: &[u8]) -> Result<Method, ParseError> {
        match s {
            b"GET" => Ok(Method::Get),
            b"POST" => Ok(Method::Post),
            b"HEAD" => Ok(Method::Head),
            b"OPTIONS" => Ok(Method::Options),
            b"PUT" => Ok(Method::Put),
            b"DELETE" => Ok(Method::Delete),
            b"TRACE" => Ok(Method::Trace),
            b"CONNECT" => Ok(Method::Connect),
            _ => Err(ParseError::new("Could not parse HTTP method"))
        }
    }
}

#[derive(Debug)]
pub struct Request<'a> {
    method:&'a [u8],
    uri:&'a [u8],
    version:Version,
}



#[derive(PartialEq,Debug,Clone)]
pub struct ParsedRequest {
    pub method:Method,
    pub original_uri:String,
    pub file_type:String,
    pub uri:PathBuf,
    pub version:Version
}

impl ParsedRequest {
    pub fn get_mime_type(&self) -> &str {
        match &self.file_type {
            s if s == "html" => "text/html",
            s if s == "jpeg" => "image/jpeg",
            s if s == "jpg" => "image/jpeg",
            s if s == "png" => "image/png",
            s if s == "js" => "text/javascript",
            s if s == "css" => "text/css",
            s if s == "gif" => "image/gif",
            s if s == "svg" => "image/svg+xml",
            s => {
                println!("{}{}:{}{}","Error:".red(),"mime-type not supported for file type", s, "feel free to add it");
                "text/html"
            }
       } 
    }
}

#[derive(PartialEq,Debug,Clone,Copy)]
pub enum Version {
  V10,
  V11,
}


fn is_token_char(i: u8) -> bool {
  is_alphanumeric(i) ||
  b"!#$%&'*+-.^_`|~".contains(&i)
}

named!(pub token, take_while!(is_token_char));
named!(pub sp<char>, char!(' '));
named!(pub crlf, tag!("\r\n"));
named!(pub vchar_1, take_while!(is_vchar));
fn is_vchar(i: u8) -> bool {
  i > 32 && i <= 126
}

named!(pub request<Request>, do_parse!(
    method: token >>
    sp>>
    uri: vchar_1 >>
    sp>>
    version: http_version>>
    crlf>> (
        Request{
            method:method,
            uri:uri,
            version:version
        }
    )
  )
);

named!(pub http_version<Version>,
do_parse!(
  tag!("HTTP/") >>
  tag!("1.") >>
  minor: one_of!("01") >> (
    if minor == '0' {
      Version::V10
    } else {
      Version::V11
    }
  )
)
);

pub fn parse_request(buf: &[u8]) -> Result<ParsedRequest, ParseError> {
    match request(buf) {
        Ok((_,r)) => {
            let method = Method::new(r.method)?;
            let path = &String::from_utf8(r.uri.to_vec())?;
            let mut request_path = std::path::Path::new(path).to_path_buf();
            let extension = std::path::Path::new(path).extension();
            let mut file_type = "html";
            
            match extension {
                None => { request_path.push("index.html"); },
                Some(ext) => { file_type = ext.to_str().unwrap(); }
            };

            let mut relative_path = std::env::current_dir().unwrap();
                        
            relative_path.push(request_path.strip_prefix("/")?);

            Ok(ParsedRequest {
                method:method,
                original_uri:path.to_string(),
                file_type:file_type.to_string(),
                uri:relative_path,
                version:r.version,
            })
        },
        _ => Err(ParseError::new("Couldn't parse the request from bytes"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! parse_test {
        ($name:ident, $request:expr, $result_path:expr) => {
            #[test]
            fn $name() {
                let result = parse_request($request.as_bytes()).unwrap();

                let path = std::env::current_dir().unwrap().join($result_path);

                assert_eq!(result.uri.as_path().to_str(), path.as_path().to_str()); 
            }
        };
    }

    parse_test!(test_base, "GET / HTTP/1.1\r\n", "index.html");
    parse_test!(test_child, "GET /hole/ HTTP/1.1\r\n", "hole/index.html");
    parse_test!(test_child_2, "GET /hole/furtherdown HTTP/1.1\r\n", "hole/furtherdown/index.html");

}
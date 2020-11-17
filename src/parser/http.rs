/// HTTP Request Parser

use std::fmt;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub enum HttpRequestMethod {
    GET,
    POST,
    ILLEGAL, // -> Ignored
}

#[derive(Debug)]
pub struct HttpRequest<'t> {
    // ref: https://github.com/lennart-bot/lhi/blob/master/src/server/request.rs
    // It provides the idea to use reference & BTreeMap to track http head entries
    
    pub method: HttpRequestMethod,
    pub url: &'t str, // use reference to avoid copying
    pub version: &'t str, // use reference to avoid copying
    pub raw_head: &'t str,
    pub raw_body: &'t str,
    pub head: BTreeMap<String, &'t str>, // Other fields in head, if necessary
    
    // ref: https://stackoverflow.com/questions/41034635/idiomatic-transformations-for-string-str-vecu8-and-u8
    // It talks about trans between vec, u8, str, etc.
}

impl fmt::Display for HttpRequest<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let req_type = match self.method {
            HttpRequestMethod::GET  => "GET",
            // HttpRequestMethod::POST => "POST",
            _ => "ILLEGAL"
        };
        write!(f, "HttpRequest:\nmethod {}\nurl {}\nversion {}", req_type, self.url, self.version)
    }
}

impl<'t> From<&'t str> for HttpRequest<'t> {
    /// Transform raw http req string to HttpRequest
    /// 
    /// Use rust's "from/into" style
    /// ```
    /// let hr = HttpRequest::from(raw_head);
    /// ```
    fn from (input: &'t str) -> Self {
        let mut lines = input.lines();

        let mut head_line = match lines.next() {
            Some(line) => line,
            None => return HttpRequest::invalidRequest(),
        };
        let mut head_line_splited = head_line.split(" ");

        let method = match head_line_splited.next() {
            Some(raw_method) => match raw_method {
                "GET"  => HttpRequestMethod::GET,
                "POST" => HttpRequestMethod::POST,
                _ => return HttpRequest::invalidRequest(),
            },
            None => return HttpRequest::invalidRequest(),
        };
        
        let url = match head_line_splited.next() {
            Some(raw_url) => raw_url,
            None => return HttpRequest::invalidRequest(),
        };
        
        let version = match head_line_splited.next() {
            Some(raw_version) => raw_version,
            None => return HttpRequest::invalidRequest(),
        };
   
        // println!("http head parse result: {}", result);
        Self {
            method: method,
            url: url,
            version: version,
            raw_head: "",
            raw_body: "",
            head: BTreeMap::new()
        }
    }
}

impl HttpRequest<'_> {
    fn invalidRequest() -> Self {
        HttpRequest {
            method: HttpRequestMethod::ILLEGAL,
            url: "",
            version: "",
            raw_head: "",
            raw_body: "",
            head: BTreeMap::new()
        }
    }
}

pub fn test(){
    println!("HttpParser imported");
}

pub fn parse_http_head(input: &str) -> Result<HttpRequest, &'static str> {
    Ok(HttpRequest::from(input))
}
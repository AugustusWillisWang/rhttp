/// HTTP Request / Response Parser

use std::fmt;
use std::collections::BTreeMap;

// Parse HTTP Request

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
    // pub raw_head: &'t str,
    // pub raw_body: &'t str,
    pub headers: BTreeMap<String, &'t str>, // Other fields in head, if necessary
    pub body: std::str::Lines<'t>,
    
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
        write!(f, "HttpRequest:\nmethod {}\nurl {}\nversion {}\nheaders {:#?}\nbody {:#?}", req_type, self.url, self.version, self.headers, self.body)
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
        
        let start_line = match lines.next() {
            Some(line) => line,
            None => return HttpRequest::invalid_request(),
        };
        let mut start_line_splited = start_line.split(" ");
        
        let method = match start_line_splited.next() {
            Some(raw_method) => match raw_method {
                "GET"  => HttpRequestMethod::GET,
                "POST" => HttpRequestMethod::POST,
                _ => return HttpRequest::invalid_request(),
            },
            None => return HttpRequest::invalid_request(),
        };
        
        let url = match start_line_splited.next() {
            Some(raw_url) => raw_url,
            None => return HttpRequest::invalid_request(),
        };
        
        let version = match start_line_splited.next() {
            Some(raw_version) => raw_version,
            None => return HttpRequest::invalid_request(),
        };

        let mut headers = BTreeMap::<String, &'t str>::new();

        // check line by line, do not stop until we can not find valid "k: v" pair
        loop {
            let mut line_splited = lines.next().unwrap_or("").split(":");
            match (line_splited.next(), line_splited.next()) {
                (Some(k), Some(v)) => {
                    headers.insert(k.trim().to_string(), v.trim());
                },
                _ => break
            }
        }

        // println!("headers BT: {:#?}", headers);
        // println!("http head parse result: {}", result);
        Self {
            method: method,
            url: url,
            version: version,
            headers: headers,
            body: lines
        }
    }
}

impl HttpRequest<'_> {
    fn invalid_request() -> Self {
        HttpRequest {
            method: HttpRequestMethod::ILLEGAL,
            url: "",
            version: "",
            headers: BTreeMap::new(),
            body: "".lines(),
        }
    }
}

// Parse HTTP Response

// TODO
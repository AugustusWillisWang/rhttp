/// HTTP Request / Response Parser

use std::fmt;
use std::collections::BTreeMap;
use std::net::TcpStream;
use std::io::BufReader;
use std::io::Lines;

use super::super::BUFFER_SIZE;

mod method;

// Parse HTTP Request

#[derive(Debug, PartialEq)]
pub enum HttpRequestMethod {
    GET,
    POST,
    HEAD,
    PUT,
    OPTIONS,
    ILLEGAL, // -> Ignored
}

pub type TcpLine = Lines<BufReader<TcpStream>>;

#[derive(Debug)]
pub struct HttpRequest<'t> {
    /// ref: https://github.com/lennart-bot/lhi/blob/master/src/server/request.rs
    /// It provides the idea to use reference & BTreeMap to track http head entries
    
    pub method: HttpRequestMethod,
    pub url: &'t str, // use reference to avoid copying
    pub version: &'t str, // use reference to avoid copying
    pub headers: BTreeMap<String, &'t str>, // Other fields in head, if necessary
    pub body: Lines<BufReader<TcpStream>>,
    // pub size: usize,
}

impl fmt::Display for HttpRequest<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let req_type = match self.method {
            HttpRequestMethod::GET     => "GET",
            HttpRequestMethod::POST    => "POST",
            HttpRequestMethod::HEAD    => "HEAD",
            HttpRequestMethod::PUT     => "PUT",
            HttpRequestMethod::OPTIONS => "OPTIONS",
            _ => "ILLEGAL"
        };
        write!(f, "HttpRequest:\nmethod {}\nurl {}\nversion {}\nheaders {:#?}", req_type, self.url, self.version, self.headers)
    }
}

impl<'t> From<Lines<BufReader<TcpStream>>> for HttpRequest<'t> {
    /// Transform raw http req string to HttpRequest
    /// 
    /// Use rust's "from/into" style
    /// ```
    /// let hr = HttpRequest::from(raw_head);
    /// ```

    fn from (mut input: Lines<BufReader<TcpStream>>) -> Self {
        let start_line = match input.next() {
            Some(line) => line,
            None => return HttpRequest::invalid_request(input),
        };
        let mut start_line_splited = start_line.unwrap().split(" "); //FIXME
        
        let method = match start_line_splited.next() {
            Some(raw_method) => match raw_method {
                "GET"     => HttpRequestMethod::GET,
                "POST"    => HttpRequestMethod::POST,
                "HEAD"    => HttpRequestMethod::HEAD,
                "PUT"     => HttpRequestMethod::PUT,
                "OPTIONS" => HttpRequestMethod::OPTIONS,
                _ => return HttpRequest::invalid_request(input),
            },
            None => return HttpRequest::invalid_request(input),
        };
        
        let url = match start_line_splited.next() {
            Some(raw_url) => raw_url,
            None => return HttpRequest::invalid_request(input),
        };
        
        let version = match start_line_splited.next() {
            Some(raw_version) => raw_version,
            None => return HttpRequest::invalid_request(input),
        };

        let mut headers = BTreeMap::<String, &'t str>::new();

        // check line by line, do not stop until we can not find valid "k: v" pair
        loop {
            let mut line_splited = input.next().unwrap_or(Ok("".to_string())).unwrap().split(":");
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
            body: input,
            // size: input.chars().count(),
        }
    }
}

impl HttpRequest<'_> {
    fn invalid_request(input: Lines<BufReader<TcpStream>>) -> Self {
        HttpRequest {
            method: HttpRequestMethod::ILLEGAL,
            url: "",
            version: "",
            headers: BTreeMap::new(),
            body: input,
            // size: 0,
        }
    }
}

// Parse HTTP Response

/// HTTP response waiting for sending
#[derive(Debug)]
pub struct HttpResponse<'t> {
    /// ref: https://github.com/lennart-bot/lhi/blob/master/src/server/request.rs
    /// It provides the idea to use reference & BTreeMap to track http head entries
    ///
    /// ref: https://developer.mozilla.org/zh-CN/docs/Web/HTTP/Messages
    /// It makes reading RFC much easier
    
    pub status_code: u32,
    pub status_text: &'t str,
    /// <String, String> instead of <String, &str> 
    pub headers: BTreeMap<String, String>, 
    /// TODO: switch to buffer
    pub body: String, 
}

impl fmt::Display for HttpResponse<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HttpResponse:\nstatus_code {}\nstatus_text {}\nheaders {:#?}", self.status_code, self.status_text, self.headers)
    }
}

impl HttpResponse<'_> {

    pub fn error_400() -> Self {
        Self {
            status_code: 400,
            status_text: "Bad Request",
            headers: BTreeMap::<String, String>::new(),
            body: "".to_string(),
        }
    }

    pub fn _error_404() -> Self {
        Self {
            status_code: 404,
            status_text: "Not Found",
            headers: BTreeMap::<String, String>::new(),
            body: "404 Not Found".to_string(),
        }
    }

    pub fn error_405() -> Self {
        Self {
            status_code: 405,
            status_text: "Method Not Allowed",
            headers: BTreeMap::<String, String>::new(),
            body: "".to_string(),
        }
    }

    pub fn error_500() -> Self {
        Self {
            status_code: 500,
            status_text: "Internal Server Error",
            headers: BTreeMap::<String, String>::new(),
            body: "Undefined Interal Error Resp Body".to_string(),
        }
    }

    pub fn error_507() -> Self {
        Self {
            status_code: 507,
            status_text: "Insufficient Storage",
            headers: BTreeMap::<String, String>::new(),
            body: "".to_string(),
        }
    }

    /// Generate HttpResponse from HttpRequest
    /// 
    /// Return Ok(HttpResponse) if a response is needed
    /// Return None if no response is required
    pub fn new(request: &mut HttpRequest, root_dir: &str) -> Option<Self> {
        // let status_code =  404;
        // let status_text = "Undefined Interal Error";
        let mut headers = BTreeMap::<String, String>::new();
        // let body = "Undefined Interal Error Resp Body";
        
        // Response Headers
        headers.insert("Server".to_string(), "rhttp".to_string());
        
        // Entity Headers
        // TODO: Content-Type

        // General Headers
        // TODO: Connection
        // TODO: Keep-Alive
        
        // body_type match ... {...}
        // TODO: Content-Type
        // TODO: Content-Length
        // TODO: Transfer-Encoding
        // Ignored: Multiple-resource bodies

        // HttpRequest match
        match request.method {
            HttpRequestMethod::ILLEGAL => {
                println!("Ignored illegal request");
                return None
            }

            HttpRequestMethod::GET => {
                method::generate_get_response(request, headers, root_dir)
            }
            
            HttpRequestMethod::POST => {
                method::generate_post_response(request, headers, root_dir)
            }
            
            HttpRequestMethod::PUT => {
                method::generate_put_response(request, headers, root_dir)
            }
            
            HttpRequestMethod::HEAD => {
                method::generate_head_response(request, headers, root_dir)
            }
            
            HttpRequestMethod::OPTIONS => {
                method::generate_options_response(request, headers, root_dir)
            }
        }
    }

    /// Generate real HTTP response from HttpResponse 
    pub fn generate_string(&self) -> String {
        let status_line = format!("HTTP/1.1 {} {}\n", self.status_code, self.status_text);
        let mut headers_str = String::new();
        // TODO: use vec, and use vec.resource(1024) to pre allocate space
        // body.size may help 
        // headers_str.resource(1024);
        for (k, v) in &self.headers {
            headers_str.push_str(&format!("{}: {}\n", k, v));
        }
        headers_str.push('\n'); // add a space line
        // read file if necessary
        String::from(format!("{}{}{}", status_line, headers_str, self.body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_get_request() {
        let raw_get = 
r"GET /favicon.ico HTTP/1.1
Host: 127.0.0.1:7878
Connection: keep-alive
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.198 Safari/537.36 Edg/86.0.622.69
DNT: 1
Accept: image/webp,image/apng,image/*,*/*;q=0.8
Sec-Fetch-Site: same-origin
Sec-Fetch-Mode: no-cors
Sec-Fetch-Dest: image
Referer: http://127.0.0.1:7878/
Accept-Encoding: gzip, deflate, br
Accept-Language: zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6
Cookie: zentaosid=g9uhstb2uthp5budvqbh6j3d0u
";
        // TODO: use real TcpStream for testing
        // let request = HttpRequest::from(raw_get);
        // assert_eq!(request.method, HttpRequestMethod::GET);
    }
}
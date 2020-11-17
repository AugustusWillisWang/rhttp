// mod rhttp;

use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use rhttp::ThreadPool;

mod parser; // parser for http head
use parser::http::*; // import http head data structure

fn main() {

    test();

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    loop {
        for stream in listener.incoming().take(2) {
            let stream = stream.unwrap();
    
            pool.execute(|| {
                handle_connection(stream);
            });
        }
    }
    
    // Should not reach here
    // println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "error/404.html")
    };

    println!("Raw request:\n{}", String::from_utf8_lossy(&buffer[..]));
    let buf_str = &String::from_utf8_lossy(&buffer[..]);
    // let request = parse_http_head(buf_str).unwrap();
    let request = HttpRequest::from(buf_str as &str);
    println!("Parsed request: {}", request);
    
    if request.method == HttpRequestMethod::ILLEGAL {
        println!("Ignored illegal request");
    }

    let page_dir = "/mnt/c/Workpath/rhttp/page/";
    let full_filename = format!("{}{}", page_dir, filename);
    println!("Return file {}\n", full_filename);

    let contents = fs::read_to_string(full_filename).unwrap();

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
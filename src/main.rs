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
            
    println!("Raw request:\n{}", String::from_utf8_lossy(&buffer[..]));
    let buf_str = &String::from_utf8_lossy(&buffer[..]);
    // let request = parse_http_head(buf_str).unwrap();
    let request = HttpRequest::from(buf_str as &str);
    println!("Parsed request: {}", request);
    
    let page_dir = "/mnt/c/Workpath/rhttp/page"; // TODO: add to config
    
    let (status_line, filename) = match request.method {
        HttpRequestMethod::ILLEGAL => {
            println!("Ignored illegal request");
            return;
        }
        HttpRequestMethod::GET => {
            // check if requsested resource exists
            if request.url == "/" {
                ("HTTP/1.1 200 OK\r\n\r\n", format!("{}/index.html", page_dir))
            } else {
                let full_filename = format!("{}{}", page_dir, request.url);
                match fs::File::open(&full_filename) {
                    Ok(_) => ("HTTP/1.1 200 OK\r\n\r\n", full_filename), // if resource exists, return it to client
                    _ => ("HTTP/1.1 404 NOT FOUND\r\n\r\n", format!("{}/error/404.html", page_dir)), // otherwise, 404
                }
            }
        }
        HttpRequestMethod::POST => {
            println!("POST is not supported for now");
            return;
        }
    };
    
    println!("Return file {}\n", filename);
    
    let contents = fs::read_to_string(filename).unwrap();
    let response = format!("{}{}", status_line, contents);
    
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
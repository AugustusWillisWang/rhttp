/// RHTTP
/// 
/// Rust HTTP Server for NW 2020

// TODO List

// * HTTP Post/Get
// * Upload
// * Download
// * HTTP分块传输
// * 支持HTTP持久连接和管道
// * Use lib to deal with HTTPS Request
// * openssl or others?

// RUST related opt
//
// * env_logger  ref: http://llever.com/CliInput-wg-zh/tutorial/output.zh.html
// * structopt for CliInput parameter parsing
// * confy for Serialize/Deserialize config

use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
// use std::thread;
// use std::time::Duration;

use rhttp::ThreadPool;
mod parser; // parser for http head
use parser::http::*; // import http head data structure

use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use structopt::StructOpt;
extern crate confy;
#[macro_use]
extern crate serde_derive;
extern crate serde;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    port: u32,
    thread_number: usize,
    root_dir: String,
}

impl Default for Config {
    fn default() -> Self { Self {
        port: 7878,
        thread_number: 4,
        root_dir: "/mnt/c/Workpath/rhttp/page".into()
    } }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "RHTTP", about = "Rust HTTP Server for NW 2020.")]
struct CliInput {
    // pattern: String,
    /// verbosity level
    #[structopt(short = "v", parse(from_occurrences), default_value = "0")]
    verbose: u32,
    /// Set port
    #[structopt(short = "p", default_value = "0")]
    port: u32,
    /// Set number of threads
    #[structopt(short = "j", default_value = "0")]
    thread_number: usize,
    #[structopt(name = "FILE", parse(from_os_str), default_value = "")]
    config_file: Vec<std::path::PathBuf>,
}

fn main() {
    let args = CliInput::from_args();
    println!("RHTTP server started.");
    println!("{:#?}", args);
    let mut cfg: Config = confy::load("config").unwrap();
    if args.port != 0 {
        cfg.port = args.port;
    }
    if args.thread_number != 0 {
        cfg.thread_number = args.thread_number;
    }
    println!("{:#?}", cfg);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", cfg.port)).unwrap();
    let pool = ThreadPool::new(cfg.thread_number);

    loop {
        for stream in listener.incoming().take(2) {
            let stream = stream.unwrap();
            let root_dir = cfg.root_dir.clone();
            pool.execute(move || {
                handle_connection(stream, &root_dir);
            });
        }
    }
    
    // Should not reach here
    // println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream, root_dir: &str) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
            
    // ref: https://stackoverflow.com/questions/60070627/does-stringfrom-utf8-lossy-allocate-memory
    // > If our byte slice is invalid UTF-8, then we need to insert the replacement characters, 
    // > which will change the size of the string, and hence, require a String. 
    // > But if it's already valid UTF-8, we don't need a new allocation. 
    // > This return type allows us to handle both cases.
    println!("Raw request:\n{}", String::from_utf8_lossy(&buffer[..]));
    let buf_str = &String::from_utf8_lossy(&buffer[..]);
    // let request = parse_http_head(buf_str).unwrap();
    let request = HttpRequest::from(buf_str as &str); // from_utf8_lossy returns a Cow<'a, str>, use as to make compiler happy
    println!("Parsed request: {}", request);
    
    let page_dir = root_dir;
    
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
                    Ok(_) => ("HTTP/1.1 200 OK\r\n\r\n", full_filename), // if resource exists, return it to CliInputent
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
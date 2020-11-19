//! RHTTP
//! 
//! Rust HTTP Server for NW 2020
//! 
//! TODO List
//! * 完善HTTP Req框架 [DONE]
//!     * 添加Header处理 [DONE]
//! * 实现HTTP Resp框架 [DONE]
//!     * 实现Parser [DONE]
//!     * 实现RESP字符串生成 [DONE]
//! * 完善方法
//!     * GET [DONE]
//!     * PUT [DONE]
//!     * POST [DONE]
//!     * HEAD [DONE]
//!     * OPTIONS [DONE]
//! * 实现Test框架
//!     * 检查各种方法的实现
//!     * 检查多线程的实现
//!     * RUST使用TCP发送数据可参考: https://blog.csdn.net/lcloveyou/article/details/105755676
//! * 详细实现POST方法中对Content-Type的支持: 支持使用POST传输文件: [DONE] 
//!     * ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type
//!     * multipart/form-data暂不准备实现 [DONE]
//!     * 文件传输测试
//! * 分块传输支持
//!     * ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Transfer-Encoding
//!     * 分块传输解析
//!         * Transfer-Encoding: chunked
//!         * New API
//! * Keep-alive
//! * Pipelined
//! * HTTPS

/// 要求列表
/// * HTTP Get [DONE]
/// * HTTP Post
/// * Upload
/// * Download
/// * HTTP分块传输
/// * 支持HTTP持久连接和管道 ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Connection_management_in_HTTP_1.x
/// * Use lib to deal with HTTPS Request
///     * openssl or others?
/// * multithread [DONE]

/// RUST related opt
///
/// * env_logger for log in different level
///     * ref: http://llever.com/CliInput-wg-zh/tutorial/output.zh.html
/// * structopt for CliInput parameter parsing [DONE]
/// * confy for Serialize/Deserialize config [DONE]
/// * refactor: let reader = BufReader::new(&stream);
/// * refactor: let mut writer = BufWriter::new(&stream);

/// HTTP Standard Reference
/// ref: https://developer.mozilla.org/en-US/docs/Web/HTTP

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
// use std::thread;
// use std::time::Duration;

mod tpool;
use tpool::*;

mod parser; // parser for http head
use parser::http::*; // import http head data structure

use openssl::ssl::{SslMethod, SslAcceptor, SslStream, SslFiletype};
use std::sync::Arc;

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

const BUFFER_SIZE: usize = 4096;

#[derive(Debug, StructOpt)]
#[structopt(name = "RHTTP", about = "Rust HTTP Server for NW 2020.")]
struct CliInput {
    // pattern: String,
    /// Update config without running the real server
    #[structopt(long = "update-config", parse(from_occurrences), default_value = "0")]
    update_config: u32,
    /// Verbosity level
    #[structopt(short = "v", parse(from_occurrences), default_value = "0")]
    verbose: u32,
    /// Set port
    #[structopt(short = "p", long = "port", default_value = "0")]
    port: u32,
    /// Set number of threads
    #[structopt(short = "j", long = "thread", default_value = "0")]
    thread_number: usize,
    /// Set server root dir
    #[structopt(short = "r", long = "root-dir", name = "server_root_dir", default_value = "")]
    root_dir: String,
}

fn main() {
    let args = CliInput::from_args();
    println!("RHTTP server started.");
    println!("{:#?}", args);
    let mut cfg: Config = confy::load("rhttp_config").unwrap();
    if args.port != 0 {
        cfg.port = args.port;
    }
    if args.thread_number != 0 {
        cfg.thread_number = args.thread_number;
    }
    if args.root_dir != "" {
        cfg.root_dir = args.root_dir.clone();
    }
    if args.update_config != 0 {
        println!("New config updated:\n{:#?}", cfg);
        confy::store("rhttp_config", cfg).unwrap();
        return
    }
    println!("{:#?}", cfg);

    // certificate loading
    let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    acceptor.set_private_key_file("/home/lfz/Pictures/rhttp/page/test2020.com_key.key", SslFiletype::PEM).unwrap();
    acceptor.set_certificate_chain_file("/home/lfz/Pictures/rhttp/page/test2020.com_chain.crt").unwrap();
    acceptor.check_private_key().unwrap();
    let acceptor = Arc::new(acceptor.build());

    let listener = TcpListener::bind(format!("127.0.0.1:{}", cfg.port)).unwrap();
    let pool = ThreadPool::new(cfg.thread_number);

    loop {
        for stream in listener.incoming().take(2) {
            match stream {
                Ok(stream) => {
                    let acceptor = acceptor.clone();
                    let stream = acceptor.accept(stream).unwrap();
                    let root_dir = cfg.root_dir.clone();
                    pool.execute(move || {
                        handle_connection(stream, &root_dir);
                    });
                }
                Err(_e) => { /* connection failed */ }
            }
        }
    }
    
    // Should not reach here
    // println!("Shutting down.");
}

fn handle_connection(mut stream: SslStream<TcpStream>, root_dir: &str) {
    let mut buffer = [0; BUFFER_SIZE];
    stream.read(&mut buffer).unwrap();
            
    // ref: https://stackoverflow.com/questions/60070627/does-stringfrom-utf8-lossy-allocate-memory
    // > If our byte slice is invalid UTF-8, then we need to insert the replacement characters, 
    // > which will change the size of the string, and hence, require a String. 
    // > But if it's already valid UTF-8, we don't need a new allocation. 
    // > This return type allows us to handle both cases.
    println!("Raw request:\n{}", String::from_utf8_lossy(&buffer[..]));
    let buf_str = &String::from_utf8_lossy(&buffer[..]);

    let mut request = HttpRequest::from(buf_str as &str); // from_utf8_lossy returns a Cow<'a, str>, use as to make compiler happy
    println!("{}", request);

    // if keep-alive is not assigned, mark Connection as close
    let mut keep_alive = true; // keep_alive is opened by default
    if let Some(connection) = request.headers.get("Connection") {
        if connection.trim() != "keep-alive" {
            keep_alive = false; // invalid header, close it
        }
    } else {
        keep_alive = false;
    }
    
    match HttpResponse::new(&mut request, root_dir) {
        Some(response) => {
            // if headers.Connection not assigned, assign it automaticly
            if let Some(resp_keep_alive) = request.headers.get("Connection") {
                keep_alive = resp_keep_alive.trim() == "keep-alive";
            } else {
                let connection_value = if keep_alive { "keep_alive" } else { "close" };
                request.headers.insert("Connection".to_string(), connection_value);
            }
            println!("{}\n", response);
            stream.write(response.generate_string().as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        _ => return // TCP will also be closed
    } 

    // if !keep_alive {
    //     StopTCP;
    // }
}

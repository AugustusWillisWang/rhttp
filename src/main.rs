//! RHTTP
//! =====
//! 
//! Rust HTTP Server for NW 2020
//! 
//! ---
//!
//! # Features
//! 
//! Methods Supported:
//! 
//! * GET
//! * PUT
//! * POST
//! * HEAD
//! * OPTIONS
//! 
//! Other features:
//! 
//! * keep-alive support
//! * chunk support
//! * multi-thread using built-in thread pool
//! * HTTPS\* (https branch)
//! 
//! # Usage
//! 
//! ```
//! rhttp [OPTIONS]
//! 
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//! 
//! OPTIONS:
//!         --load-config <load-config>...        Use config [default: 0]
//!     -p, --port <port>                         Set port [default: 0]
//!     -r, --root-dir <server_root_dir>          Set server root dir [default: ]
//!     -j, --thread <thread-number>              Set number of threads [default: 0]
//!     -t, --timeout <timeout>                   Set timeout limit [default: -1]
//!         --update-config <update-config>...    Update config without running the real server [default: 0]
//!     -v <verbose>...                           Verbosity level [default: 0]
//! ```
//! 
//! HTML files should be placed in `/page` dir.
//! 
//! # TODO List
//! 
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
//! * 实现Test框架 [DONE]
//!     * 检查各种方法的实现 [DONE]
//!     * 检查多线程的实现 [DONE]
//!     * RUST使用TCP发送数据可参考: https://blog.csdn.net/lcloveyou/article/details/105755676
//! * 详细实现POST方法中对Content-Type的支持: 支持使用POST传输文件: [DONE] 
//!     * ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type
//!     * multipart/form-data [DONE]
//!     * 文件传输测试 [DONE]
//! * 分块传输支持 [MORE TEST NEEDED]
//!     * ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Transfer-Encoding
//!     * 分块传输解析 [DONE]
//!         * Transfer-Encoding: chunked [DONE]
//! * Keep-alive [DONE]
//! * Pipelined
//! * HTTPS [DONE] (incompatible with HTTP)

/// 要求列表
/// * HTTP Get [DONE]
/// * HTTP Post [DONE]
/// * Upload [DONE]
/// * Download [DONE]
/// * HTTP分块传输 [NEED UPDATE]
/// * 支持HTTP持久连接 [DONE]
///     ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Connection_management_in_HTTP_1.x
/// * 支持HTTP持久连接管道 [DELAYED]
/// * Use lib to deal with HTTPS Request
///     * openssl [DONE]
///     * 浏览器兼容性问题
///         * 测试时用`firefox`吧
/// * multithread [DONE]

/// RUST related opt
///
/// * env_logger for log in different level
///     * ref: http://llever.com/CliInput-wg-zh/tutorial/output.zh.html
/// * structopt for CliInput parameter parsing [DONE]
/// * confy for Serialize/Deserialize config [DONE]
/// * refactor: let reader = BufReader::new(&stream); [IGNORED]
/// * refactor: let mut writer = BufWriter::new(&stream); [IGNORED]

/// HTTP Standard Reference
/// ref: https://developer.mozilla.org/en-US/docs/Web/HTTP
/// ref: https://tools.ietf.org/html/rfc7230

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
// use std::thread;
// use std::time::Duration;
// use std::rc::Rc;

pub mod tpool;
pub use tpool::*;

pub mod parser; // parser for http head
pub use parser::http::*; // import http head data structure

use parser::http::method::utils::chunk::*;

use openssl::ssl::{SslMethod, SslAcceptor, SslStream, SslFiletype};
use std::sync::Arc;

// use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use structopt::StructOpt;
extern crate confy;
#[macro_use]
extern crate serde_derive;
extern crate serde;

/// Request buffer size
pub const BUFFER_SIZE: usize = 32768;
/// Default page root path
// pub const DEFAULT_ROOT: &str = "/mnt/c/Workpath/rhttp/page";
pub const DEFAULT_ROOT: &str = "/home/lfz/Videos/rhttp/page";

/// Global config file, shared by all threads
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// port binging
    port: u32,
    /// max number of threads created in the thread pool
    thread_number: usize,
    /// file root dir
    root_dir: String, 
    /// timeout unit: secs
    timeout: i64, 
    /// enable chunk resp, chunk req is always supported
    chunk: bool, 
}

impl Default for Config {
    fn default() -> Self { Self {
        port: 7878,
        thread_number: 4,
        root_dir: DEFAULT_ROOT.into(),
        timeout: 1,
        chunk: false,
    } }
}

/// Command-line parameters
#[derive(Debug, StructOpt)]
#[structopt(name = "RHTTP", about = "Rust HTTP Server for NW 2020.")]
struct CliInput {
    // pattern: String,
    /// Update config without running the real server
    #[structopt(long = "update-config", parse(from_occurrences), default_value = "0")]
    update_config: u32,
    /// Use config
    #[structopt(long = "load-config", parse(from_occurrences), default_value = "0")]
    load_config: u32,
    /// Verbosity level
    #[structopt(short = "v", parse(from_occurrences), default_value = "0")]
    verbose: u32,
    /// Set port
    #[structopt(short = "p", long = "port", default_value = "0")]
    port: u32,
    /// Set number of threads
    #[structopt(short = "j", long = "thread", default_value = "0")]
    thread_number: usize,
    /// Set timeout limit
    #[structopt(short = "t", long = "timeout", default_value = "-1")]
    timeout: i64,
    /// Set server root dir
    #[structopt(short = "r", long = "root-dir", name = "server_root_dir", default_value = "")]
    root_dir: String,
}

/// Entry
fn main() {
    // setup config
    let args = CliInput::from_args();
    println!("RHTTP server started.");
    println!("{:#?}", args);
    let mut cfg: Config = if args.load_config != 0 {
        confy::load("rhttp_config").unwrap()
    } else {
        Config::default()
    };
    if args.port != 0 {
        cfg.port = args.port;
    }
    if args.thread_number != 0 {
        cfg.thread_number = args.thread_number;
    }
    if args.timeout != -1 {
        cfg.timeout = args.timeout;
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
    acceptor.set_private_key_file(format!("{}/test2020.com_key.key", cfg.root_dir), SslFiletype::PEM).unwrap();
    acceptor.set_certificate_chain_file(format!("{}/test2020.com_chain.crt", cfg.root_dir)).unwrap();
    acceptor.check_private_key().unwrap();
    let acceptor = Arc::new(acceptor.build());

    // prepare TCP port and thread pool
    let listener = TcpListener::bind(format!("127.0.0.1:{}", cfg.port)).unwrap();
    let pool = ThreadPool::new(cfg.thread_number);
    
    // when new TCP request incomes, handle_connection
    loop {
        for stream in listener.incoming().take(2) {
            match stream {
                Ok(stream) => {
                    let acceptor = acceptor.clone();
                    let stream = acceptor.accept(stream).unwrap();
                    let cfg_cp = cfg.clone();
                    pool.execute(move || {
                        handle_connection(stream, cfg_cp);
                    });
                }
                Err(_e) => { /* connection failed */ }
            }
        }
    }
    
    // Should not reach here
    // println!("Shutting down.");
}

/// Main function to handle http connection
/// 
/// When a new TCP link established, give it to handle_connection in a free worker.
/// 
/// Returning from this function will close TCP link.

fn handle_connection(mut stream: SslStream<TcpStream>, cfg: Config) {
    let root_dir: &str = &cfg.root_dir;
    let timeout: u64 = cfg.timeout as u64;
    loop{
        let mut buffer = [0; BUFFER_SIZE];
        match stream.read(&mut buffer) {
            Err(_) => { 
                // TCP timeout, close TCP link
                println!("keep-alive timeout, close TCP link.");
                return 
            } 
            _ => {}
        }
        
        // ref: https://stackoverflow.com/questions/60070627/does-stringfrom-utf8-lossy-allocate-memory
        // > If our byte slice is invalid UTF-8, then we need to insert the replacement characters, 
        // > which will change the size of the string, and hence, require a String. 
        // > But if it's already valid UTF-8, we don't need a new allocation. 
        // > This return type allows us to handle both cases.
        println!("Raw request:\n{}", String::from_utf8_lossy(&buffer[..]));
        let buf_str = &String::from_utf8_lossy(&buffer[..]);
        
        // parse http request
        let mut request = HttpRequest::from(buf_str as &str); // from_utf8_lossy returns a Cow<'a, str>, use as to make compiler happy
        // println!("{}", request);
        
        // if keep-alive is not assigned, mark Connection as close
        let mut keep_alive = true; // keep_alive is opened by default
        if let Some(connection) = request.headers.get("Connection") {
            if connection.to_lowercase() != "keep-alive" {
                keep_alive = false; // invalid header, close it
            }
        } else {
            keep_alive = false;
        }
        
        // generate http response according to require type
        match HttpResponse::new(&mut request, &cfg) {

            Some(mut response) => {
                // setup Keep-Alive: timeout
                response.headers.insert("Keep-Alive".to_string(), format!("timeout={}", timeout));
                // if headers.Connection not assigned, assign it automaticly
                if let Some(resp_keep_alive) = response.headers.get("Connection") {
                    keep_alive = resp_keep_alive.to_lowercase() == "keep-alive";
                } else {
                    let connection_value = if keep_alive { "keep-alive" } else { "close" };
                    response.headers.insert("Connection".to_string(), connection_value.to_string());
                    println!("keep_alive: {}", keep_alive);
                }
                println!("{}\n", response);
 
                // raw_resp_body :Vec<u8>
                // TODO: enable chunk resp (15 mins of work?)
                let raw_resp_body = if response.need_send_raw_file() {
                    let filename = format!("{}/{}", root_dir, request.url);
                    match std::fs::read(filename) {
                        Ok(i) => {
                            println!("sending binary file");
                            response.headers.insert("Content-Length".to_string(), i.len().to_string());
                            i
                        },
                        Err(_) => {
                            // File may be removed by other threads, just ignore it or return
                            println!("fail to re-read binary file");
                            return
                        } 
                    }
                } else {
                    let res = response.body.as_ref().unwrap().clone().into_bytes(); // FIXME: copy, perf loss
                    // calculate raw body len
                    response.headers.insert("Content-Length".to_string(), res.len().to_string());
                    res
                };

                // generate final headers
                let resp_string = response.generate_head_string();

                println!("resp content head:\n{}\n", resp_string);
                stream.write(resp_string.as_bytes()).unwrap();
                stream.write(&raw_resp_body).unwrap();
                stream.flush().unwrap();
                println!("response send at {}.", std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs());
                if !keep_alive {
                    return;
                } else {
                    // if there is still content in buffer and pipleine enabled
                    // generate new HttpRequest, redo
                    // TODO

                    // otherwise, setup tcp timeout and wait
                    stream.get_ref().set_read_timeout(Some(std::time::Duration::new(timeout, 0))).unwrap();
                }
            }
            _ => return // TCP will also be closed
        } 
    }
}
    
#[cfg(test)]
mod tests {
    /// # Tips
    /// 
    /// Run
    /// ```
    /// cargo test -- --nocapture --test <test_name>
    /// ```
    /// to check response in console.
    /// 
    /// For example:
    /// 
    /// ```
    /// cargo test -- --nocapture --test post_test
    /// ```
    
    use super::*;

    /// Generate response string accoding to input request string
    /// 
    /// It is a copy of handle_connection, we use &str as input. 
    /// Thus we do not to deal with TCP in this unittest.
    /// 
    /// * All TCP-related ops are removed.
    /// * Keep-Alive will be ignored.
    fn resp_from_req_str(input: &str) -> String {
        // parse http request
        let mut request = HttpRequest::from(input);
        
        // if keep-alive is not assigned, mark Connection as close
        let mut keep_alive = true; // keep_alive is opened by default
        if let Some(connection) = request.headers.get("Connection") {
            if connection.to_lowercase() != "keep-alive" {
                keep_alive = false; // invalid header, close it
            }
        } else {
            keep_alive = false;
        }
        
        let cfg: Config = Default::default();

        // generate http response according to require type
        match HttpResponse::new(&mut request, &cfg) {
            Some(mut response) => {
                // setup Keep-Alive: timeout
                response.headers.insert("Keep-Alive".to_string(), format!("timeout={}", 4));
                // if headers.Connection not assigned, assign it automaticly
                if let Some(_) = response.headers.get("Connection") {
                } else {
                    let connection_value = if keep_alive { "keep_alive" } else { "close" };
                    response.headers.insert("Connection".to_string(), connection_value.to_string());
                    println!("keep_alive: {}", keep_alive);
                }
                println!("{}\n", response);
                println!("response generated at {}.", std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs());
                return response.generate_string()
            }
            _ => {
                panic!("server rejected to generate response, tcp cloned");
                // return "ERROR".to_string()
            }
        }
    }

    /// Test basic GET method
    #[test]
    fn get_test () {
        let raw_req = 
        r"GET / HTTP/1.1
        Host: developer.mozilla.org
        Accept-Language: fr
        ";
        let raw_resp = resp_from_req_str(&raw_req);
        println!("-----\n{}\n-----\n", raw_resp);
    }
    
    /// Test basic POST method
    #[test]
    fn post_test () {
        let raw_req = 
        r"POST /contact_form.php HTTP/1.1
Host: developer.mozilla.org
Content-Length: 64
Content-Type: application/x-www-form-urlencoded
        
name=Joe%20User&request=Send%20me%20one%20of%20your%20catalogue
        ";
        let raw_resp = resp_from_req_str(&raw_req);
        println!("-----\n{}\n-----\n", raw_resp);
    }
    
    /// Use POST method to upload a file
    #[test]
    fn post_file_test () {
        let raw_req = 
    r"POST /data_tobe_send.txt HTTP/1.1
Host: developer.mozilla.org
Content-Length: 64
Content-Type: text/plain

name=Joe%20User&request=Send%20me%20one%20of%20your%20catalogue
";
        let raw_resp = resp_from_req_str(&raw_req);
        println!("-----\n{}\n-----\n", raw_resp);
    }
}
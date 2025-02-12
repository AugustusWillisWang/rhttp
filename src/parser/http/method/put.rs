use std::fs;
use std::collections::BTreeMap;

use super::super::BUFFER_SIZE;
use super::super::*;
use crate::Config;

/// Generate HttpResponse for PUT method
/// 
/// Some final work is done by `handle_connection`
/// 
/// * Return `Some(HttpResponse)` if a http response is required.
/// * Return `None` will close the TCP link or do nothing.
pub fn generate_put_response<'t>(request: &mut HttpRequest, headers: BTreeMap::<String, String>, cfg: &Config) -> Option<HttpResponse<'t>> {
    let root_dir: &str = &cfg.root_dir;

    let raw_length = match request.headers.get("Content-length") {
        Some(i) => i,
        None => return None,
    };
    let length = match raw_length.parse::<usize>() {
        Ok(i) => i,
        Err(_) => return None,
    };

    // length check
    if length >= BUFFER_SIZE {
        return Some(HttpResponse::error_507())
    }
    // get content from request
    let mut content = String::new();
    let mut newline = false;
    loop {
        if let Some(i) = request.body.next() {
            if newline { content.push_str("\n") };
            content.push_str(i);
            newline = true;
        } else {
            break;
        }
    }
    let filename = format!("{}/{}", root_dir, request.url);
    match fs::File::open(&filename) {
        Ok(_) => {
            // if resource exists, try to update it
            match fs::write(&filename, content) {
                Ok(_) => {
                    return Some( HttpResponse {
                        status_code: 200,
                        status_text: "OK",
                        headers: headers,
                        body: Some(format!("Content-Location: {}", request.url).to_string()),
                    })
                }
                _ => {
                    return Some(HttpResponse::error_500())
                }
            }
        } 
        // if resource dose not exist, create it
        _ => {
            match fs::write(&filename, content) {
                Ok(_) => {
                    return Some( HttpResponse {
                        status_code: 201,
                        status_text: "Created",
                        headers: headers,
                        body: Some(format!("Content-Location: {}", request.url).to_string()),
                    })
                }
                _ => {
                    return Some(HttpResponse::error_500())
                }
            }
        }
    }
}
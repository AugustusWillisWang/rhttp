use std::fs;
use std::collections::BTreeMap;

use super::super::BUFFER_SIZE;
use super::super::*;

pub fn generate_put_response<'t>(request: &mut HttpRequest, headers: BTreeMap::<String, String>, root_dir: &str) -> Option<HttpResponse<'t>> {
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
        if let Some(Ok(i)) = request.body.next() {
            if newline { content.push_str("\n") };
            content.push_str(&i);
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
                        body: format!("Content-Location: {}", request.url).to_string(),
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
                        body: format!("Content-Location: {}", request.url).to_string(),
                    })
                }
                _ => {
                    return Some(HttpResponse::error_500())
                }
            }
        }
    }
}
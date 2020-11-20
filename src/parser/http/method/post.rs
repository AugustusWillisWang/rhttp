use std::fs;
use std::collections::BTreeMap;

use super::super::BUFFER_SIZE;
use super::super::*;
use super::utils::chunk::*;
use crate::Config;

/// Generate HttpResponse for POST method
/// 
/// Some final work is done by `handle_connection`
/// 
/// * Return `Some(HttpResponse)` if a http response is required.
/// * Return `None` will close the TCP link or do nothing.
/// 
/// Extra code/function can be added to deal with post request body.
pub fn generate_post_response<'t>(request: &mut HttpRequest, headers: BTreeMap::<String, String>, cfg: &Config) -> Option<HttpResponse<'t>> {
    let root_dir: &str = &cfg.root_dir;

    let raw_length = match request.headers.get("Content-Length") {
        Some(i) => i,
        None => return None,
    };
    let length = match raw_length.parse::<usize>() {
        Ok(i) => i,
        Err(_) => {
            println!("len parse failed");
            return None
        },
    };
    
    // length check
    if length >= BUFFER_SIZE {
        return Some(HttpResponse::error_507())
    }
    
    let content_type = match request.headers.get("Content-Type") {
        Some(i) => i,
        _ => return Some(HttpResponse::error_400())
    };
    let filename = format!("{}/{}", root_dir, request.url);
    let chunked = match headers.get("Transfer-Encoding") {
        Some(i) => i == "chunked",
        _ => false
    };
    match content_type {
        &"application/x-www-form-urlencoded" => {
            let mut content = BTreeMap::<String, String>::new();
            
            // FIXME: reundent code
            if chunked { // FIXME: perf loss for runtime copying
                let recovered_string = chunklines_to_string(&mut request.body);
                let mut lines = recovered_string.lines();
                loop {
                    if let Some(line) = lines.next() {
                        let kv_pair = line.split("&");
                        for i in kv_pair {
                            let mut j = i.split("=");
                            if let Some(k) = j.next() {
                                if let Some(v) = j.next() {
                                    content.insert(k.to_string(), v.to_string());
                                }
                            }
                        }
                    } else {
                        break;
                    }
                }
            } else {
                loop {
                    if let Some(line) = request.body.next() {
                        let kv_pair = line.split("&");
                        for i in kv_pair {
                            let mut j = i.split("=");
                            if let Some(k) = j.next() {
                                if let Some(v) = j.next() {
                                    content.insert(k.to_string(), v.to_string());
                                }
                            }
                        }
                    } else {
                        break;
                    }
                }
            };

            // TODO: you can add extra data process here 
            println!("received kvpairs from POST: {:#?}", content);
            
            return Some( HttpResponse {
                status_code: 200,
                status_text: "OK",
                headers: headers,
                body: Some("".to_string()),
            })
        },
        &"text/plain" => {
            // transfer data to server
            // get content from request
            let mut content = String::new();
            let mut newline = false;
            if chunked {
                content = chunklines_to_string(&mut request.body);
            } else {
                loop {
                    if let Some(i) = request.body.next() {
                        if newline { content.push_str("\n") };
                        content.push_str(i);
                        newline = true;
                    } else {
                        break;
                    }
                }
            }
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
        },
        x => {
            if !x.starts_with("multipart/form-data") {
                return Some(HttpResponse::error_405())
            } else {
                // get boundry
                let mut value_splited = x.split("boundry=");
                let boundry = match value_splited.next() {
                    Some(_) => {
                        match value_splited.next() {
                            Some(i) => i,
                            _ => return Some(HttpResponse::error_400()) 
                        }
                    },
                    _ => return Some(HttpResponse::error_400())
                };
                let mut content = Vec::<String>::new();
                let mut new_part = String::new();

                // FIXME: reundent code
                if chunked { // FIXME: perf loss for runtime copying
                    let recovered_string = chunklines_to_string(&mut request.body);
                    let mut lines = recovered_string.lines();
                    while let Some(line) = lines.next() {
                        if line.starts_with(&format!("--{}--", boundry)) {
                            break
                        }
                        if line.starts_with(&format!("--{}", boundry)) && new_part.chars().count() > 0 {
                            content.push(new_part.clone()); // content get old new_part 
                            new_part = String::new();
                        } else {
                            new_part.push_str(line);
                            new_part.push_str("\n");
                        }
                    }
                } else {
                    while let Some(line) = request.body.next() {
                        if line.starts_with(&format!("--{}--", boundry)) {
                            break
                        }
                        if line.starts_with(&format!("--{}", boundry)) && new_part.chars().count() > 0 {
                            content.push(new_part.clone()); // content get old new_part 
                            new_part = String::new();
                        } else {
                            new_part.push_str(line);
                            new_part.push_str("\n");
                        }
                    }
                }
                
                // TODO: you can add extra data process here 
                println!("received multipart from POST: {:#?}", content);
                
                return Some( HttpResponse {
                    status_code: 200,
                    status_text: "OK",
                    headers: headers,
                    body: Some("".to_string()),
                })
            }
        }
    }
}
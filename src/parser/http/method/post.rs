use std::fs;
use std::fmt;
use std::collections::BTreeMap;

use super::super::BUFFER_SIZE;
use super::super::*;

pub fn generate_post_response<'t>(request: &mut HttpRequest, mut headers: BTreeMap::<String, String>, root_dir: &str) -> Option<HttpResponse<'t>> {
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
    
    let content_type = match request.headers.get("Content-Type") {
        Some(i) => i,
        _ => return Some(HttpResponse::error_400())
    };
    let filename = format!("{}/{}", root_dir, request.url);
    match content_type {
        &"application/x-www-form-urlencoded" => {
            let mut content = BTreeMap::<String, &str>::new();
            loop {
                if let Some(line) = request.body.next() {
                    let kv_pair = line.split("&");
                    for i in kv_pair {
                        let mut j = i.split("=");
                        if let Some(k) = j.next() {
                            if let Some(v) = j.next() {
                                content.insert(k.to_string(), v);
                            }
                        }
                    }
                } else {
                    break;
                }
            }

            // TODO: you can add extra data process here 
            println!("received kvpairs from POST: {:#?}", content);
            
            return Some( HttpResponse {
                status_code: 200,
                status_text: "OK",
                headers: headers,
                body: "".to_string(),
            })
        },
        &"text/plain" => {
            // transfer data to server
            // get content from request
            let mut content = String::new();
            loop {
                if let Some(i) = request.body.next() {
                    content = content + i.clone(); // FIXME
                } else {
                    break;
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
                while let Some(line) = request.body.next() {
                    if line.starts_with(&format!("--{}--", boundry)) {
                        break
                    }
                    if line.starts_with(&format!("--{}", boundry)) && new_part.chars().count() > 0 {
                        content.push(new_part.clone()); // content get old new_part 
                        new_part = String::new();
                    } else {
                        new_part = new_part + line;
                    }
                }
                
                // TODO: you can add extra data process here 
                println!("received multipart from POST: {:#?}", content);
                
                return Some( HttpResponse {
                    status_code: 200,
                    status_text: "OK",
                    headers: headers,
                    body: "".to_string(),
                })
            }
        }
    }
}
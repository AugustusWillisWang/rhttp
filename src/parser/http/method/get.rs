use std::fs;
use std::collections::BTreeMap;

use super::super::*;

// use super::utils::chunk::*;

pub fn generate_get_response<'t>(request: &mut HttpRequest, mut headers: BTreeMap::<String, String>, root_dir: &str) -> Option<HttpResponse<'t>> {
    // Sending body/payload in a GET request may cause some existing
    // implementations to reject the request — while not prohibited 
    // by the specification, the semantics are undefined. 
    // ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/GET

    // Warning should be raised if a GET request contains body/payload 
    // if request.body.next() != Some("") {
    //     println!("warning: GET request contains body/payload");
    // } else if request.body.next() != None {
    //     println!("warning: GET request contains body/payload");
    // }

    // check if requsested resource exists
    let filename = if request.url == "/" {
        format!("{}/index.html", root_dir)
    } else {
        format!("{}/{}", root_dir, request.url)
    };
    match fs::File::open(&filename) {
        // if resource exists, return 200
        Ok(_) => {
            let body = match fs::read_to_string(&filename) {
                Ok(s) => s,
                Err(_) => {
                    // transfer binary
                    return Some( HttpResponse {
                        status_code: 200,
                        status_text: "OK",
                        headers: headers,
                        body: None, // read body from raw file outside
                    })
                }
            };
            let content_length = body.chars().count();
            // println!("{}", body);
            
            // chunk resp is not enabled by default 
            let chunked = false;
            // if chunked {
                //     // chunk resp, if needed
                //     return Some( HttpResponse {
                    //         status_code: 200,
                    //         status_text: "OK",
                    //         headers: headers,
                    //         body: Some(string_to_chunk(&body)),
                    //     })
                    // } else {
            if !chunked {
                headers.insert("Content-Length".to_string(), content_length.to_string());
            }
            return Some( HttpResponse { // chunklize was moved outsize
                status_code: 200,
                status_text: "OK",
                headers: headers,
                body: Some(body),
            })
            // }
        } 
        // if resource dose not exist, return 404
        _ => {
            let body = fs::read_to_string(format!("{}/error/404.html", root_dir)).unwrap();
            headers.insert("Content-Length".to_string(), body.chars().count().to_string());
            return Some( HttpResponse {
                status_code: 404,
                status_text: "NOT FOUND",
                headers: headers,
                body: Some(body),
            })
        }
    }
}
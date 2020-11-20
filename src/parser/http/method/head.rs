use std::fs;
use std::collections::BTreeMap;

use super::super::*;
use crate::Config;

/// Generate HttpResponse for HEAD method
/// 
/// Some final work is done by `handle_connection`
/// 
/// * Return `Some(HttpResponse)` if a http response is required.
/// * Return `None` will close the TCP link or do nothing.
pub fn generate_head_response<'t>(request: &mut HttpRequest, headers: BTreeMap::<String, String>, cfg: &Config) -> Option<HttpResponse<'t>> {
    let root_dir: &str = &cfg.root_dir;

    // almost the same as GET
    // check if requsested resource exists
    let filename = if request.url == "/" {
        format!("{}/index.html", root_dir)
    } else {
        format!("{}/{}", root_dir, request.url)
    };
    match fs::File::open(&filename) {
        // if resource exists, return 200
        Ok(_) => {
            let body = fs::read_to_string(&filename).unwrap();
            return Some( HttpResponse {
                status_code: 200,
                status_text: "OK",
                headers: headers,
                body: Some("".to_string()),
            })
        } 
        // if resource dose not exist, return 404
        _ => {
            let body = fs::read_to_string(format!("{}/error/404.html", root_dir)).unwrap();
            return Some( HttpResponse {
                status_code: 404,
                status_text: "NOT FOUND",
                headers: headers,
                body: Some("".to_string()),
            })
        }
    }
}
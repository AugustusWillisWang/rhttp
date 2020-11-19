use std::fs;
use std::fmt;
use std::collections::BTreeMap;

use super::super::BUFFER_SIZE;
use super::super::*;

pub fn generate_options_response<'t>(request: &mut HttpRequest, mut headers: BTreeMap::<String, String>, root_dir: &str) -> Option<HttpResponse<'t>> {
    // ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/OPTIONS
    headers.insert("Allow".to_string(), "OPTIONS, GET, PUT, POST, HEAD".to_string());
    return Some( HttpResponse {
        status_code: 204,
        status_text: "No Content",
        headers: headers,
        body: "".to_string(),
    })
}
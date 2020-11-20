use std::fs;
use std::fmt;
use std::collections::BTreeMap;

use super::super::BUFFER_SIZE;
use super::super::*;

/// Generate HttpResponse for X method
/// 
/// Some final work is done by `handle_connection`
/// 
/// * Return `Some(HttpResponse)` if a http response is required.
/// * Return `None` will close the TCP link or do nothing.
/// 
/// You can clone this file to introduce new methods to server.
pub fn generate_get_response<'t>(request: &mut HttpRequest, mut headers: BTreeMap::<String, String>, root_dir: &str) -> Option<HttpResponse<'t>> {
    Some( HttpResponse {
        status_code: 200,
        status_text: "OK",
        headers: headers,
        body: Some("body".to_string()),
    }
}
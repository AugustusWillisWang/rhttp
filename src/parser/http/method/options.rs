use std::collections::BTreeMap;

use super::super::*;

/// Generate HttpResponse for OPTIONS method
/// 
/// Some final work is done by `handle_connection`
/// 
/// * Return `Some(HttpResponse)` if a http response is required.
/// * Return `None` will close the TCP link or do nothing.
pub fn generate_options_response<'t>(_request: &mut HttpRequest, mut headers: BTreeMap::<String, String>, _root_dir: &str) -> Option<HttpResponse<'t>> {
    // ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/OPTIONS
    headers.insert("Allow".to_string(), "OPTIONS, GET, PUT, POST, HEAD".to_string());
    return Some( HttpResponse {
        status_code: 204,
        status_text: "No Content",
        headers: headers,
        body: Some("".to_string()),
    })
}
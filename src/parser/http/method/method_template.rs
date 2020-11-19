use std::fs;
use std::fmt;
use std::collections::BTreeMap;

use super::super::BUFFER_SIZE;
use super::super::*;

pub fn generate_get_response<'t>(request: &mut HttpRequest, mut headers: BTreeMap::<String, String>, root_dir: &str) -> Option<HttpResponse<'t>> {
    Some( HttpResponse {
        status_code: 200,
        status_text: "OK",
        headers: headers,
        body: "body".to_string(),
    }
}
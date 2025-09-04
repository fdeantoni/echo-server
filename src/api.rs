use bytes::Bytes;
use serde::Serialize;
use warp::{hyper::{HeaderMap, Method}, path::FullPath};
use askama::Template;


#[derive(Serialize)]
pub struct EchoResponse {
    method: String,
    headers: Vec<(String, String)>,
    path: String,
    #[serde(skip_serializing_if="Option::is_none")]
    body: Option<String>,
    server: String
}

impl EchoResponse {
    pub fn new(method: Method, headers: HeaderMap, path: FullPath, bytes: Bytes, server: String) -> Self {
        let method = method.to_string();
        let headers = headers.iter()
            .map( |(k,v)| (k.to_string(), String::from_utf8_lossy(v.as_bytes()).to_string()) ).collect();
        let path = path.as_str().to_string();
        let body = Some(String::from_utf8_lossy(&bytes).to_string()).filter(|s| !s.is_empty() );
        EchoResponse {
            method,
            headers,
            path,
            body,
            server
        }
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    headers: Vec<(String, String)>,
    server: String
}

impl IndexTemplate {
    pub fn new(headers: HeaderMap, server: String) -> Self {
        let headers = headers.iter()
            .map( |(k,v)| (k.to_string(), String::from_utf8_lossy(v.as_bytes()).to_string()) ).collect();
        IndexTemplate {
            headers,
            server
        }
    }
}
use bytes::Bytes;
use serde::Serialize;
use std::net::SocketAddr;
use warp::{hyper::{HeaderMap, Method}, path::FullPath};
use askama::Template;


#[derive(Serialize)]
pub struct EchoResponse {
    source: String,
    method: String,
    headers: Vec<(String, String)>,
    path: String,
    #[serde(skip_serializing_if="Option::is_none")]
    body: Option<String>
}

impl EchoResponse {
    pub fn new(addr: Option<SocketAddr>, method: Method, headers: HeaderMap, path: FullPath, bytes: Bytes) -> Self {
        let source = addr.map(|sa| format!("{}:{}", sa.ip(), sa.port())).unwrap_or_else(|| "unknown".to_string());
        let method = method.to_string();
        let headers = headers.iter()
            .map( |(k,v)| (k.to_string(), String::from_utf8_lossy(v.as_bytes()).to_string()) ).collect();
        let path = path.as_str().to_string();
        let body = Some(String::from_utf8_lossy(&bytes).to_string()).filter(|s| !s.is_empty() );
        EchoResponse {
            source,
            method,
            headers,
            path,
            body
        }
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    source: String,
    headers: Vec<(String, String)>,
}

impl IndexTemplate {
    pub fn new(addr: Option<SocketAddr>, headers: HeaderMap) -> Self {
        let source = addr.map(|sa| format!("{}:{}", sa.ip(), sa.port())).unwrap_or_else(|| "unknown".to_string());
        let headers = headers.iter()
            .map( |(k,v)| (k.to_string(), String::from_utf8_lossy(v.as_bytes()).to_string()) ).collect();
        IndexTemplate {
            source,
            headers
        }
    }
}
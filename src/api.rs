use serde::Serialize;
use std::net::SocketAddr;
use warp::{hyper::HeaderMap, path::FullPath};
use askama::Template;


#[derive(Serialize)]
pub struct EchoResponse {
    source: Option<SocketAddr>,
    headers: Vec<(String, String)>,
    path: String
}

impl EchoResponse {
    pub fn new(addr: Option<SocketAddr>, headers: HeaderMap, path: FullPath) -> Self {
        let headers = headers.iter()
            .map( |(k,v)| (k.to_string(), String::from_utf8_lossy(v.as_bytes()).to_string()) ).collect();
        let path = path.as_str().to_string();
        EchoResponse {
            source: addr,
            headers,
            path
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
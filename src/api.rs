use serde::Serialize;
use std::net::SocketAddr;
use warp::{hyper::HeaderMap, path::FullPath};

#[derive(Serialize)]
pub struct Response {
    source: Option<SocketAddr>,
    headers: Vec<(String, String)>,
    path: String
}

impl Response {
    pub fn new(addr: Option<SocketAddr>, headers: HeaderMap, path: FullPath) -> Self {
        let headers = headers.iter()
            .map( |(k,v)| (k.to_string(), String::from_utf8_lossy(v.as_bytes()).to_string()) ).collect();
        let path = path.as_str().to_string();
        Response {
            source: addr,
            headers,
            path
        }
    }
}

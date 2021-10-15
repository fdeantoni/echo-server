use serde::Serialize;
use std::net::SocketAddr;
use warp::hyper::HeaderMap;

#[derive(Serialize)]
pub struct Response {
    source: Option<SocketAddr>,
    headers: Vec<(String, String)>
}

impl Response {
    pub fn new(addr: Option<SocketAddr>, headers: HeaderMap) -> Self {
        let headers = headers.iter()
            .map( |(k,v)| (k.to_string(), String::from_utf8_lossy(v.as_bytes()).to_string()) ).collect();
        Response {
            source: addr,
            headers
        }
    }
}


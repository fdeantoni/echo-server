use std::{convert::Infallible, net::SocketAddr};

use bytes::Bytes;
use warp::{Reply, hyper::{HeaderMap, Method, StatusCode}, path::FullPath};

use crate::{api, metrics};


pub async fn ok(method: Method, path: FullPath, remote: Option<SocketAddr>, headers: HeaderMap, bytes: Bytes) -> Result<impl Reply, Infallible> {
    let reply = echo(method, path, remote, headers, bytes, StatusCode::OK);
    Ok(reply)
}

pub async fn not_found(method: Method, path: FullPath, remote: Option<SocketAddr>, headers: HeaderMap, bytes: Bytes) -> Result<impl Reply, Infallible> {
    let reply = echo(method, path, remote, headers, bytes, StatusCode::NOT_FOUND);
    Ok(reply)
}

fn echo(method: Method, path: FullPath, remote: Option<SocketAddr>, headers: HeaderMap, bytes: Bytes, status: StatusCode) -> impl Reply {
    let metric_counter = metrics::ECHO_COUNT
        .get_metric_with_label_values(&[method.as_str()])
        .unwrap();
    let result = api::EchoResponse::new(remote, method, headers, path, bytes);
    let response = warp::reply::json(&result);
    metric_counter.inc();
    warp::reply::with_status(response, status)
}
use std::{convert::Infallible, net::SocketAddr};

use bytes::Bytes;
use warp::{Filter, Reply, filters::BoxedFilter, hyper::{HeaderMap, Method, StatusCode}, path::FullPath};

use askama::Template;

use crate::{api, metrics};


async fn ok(method: Method, path: FullPath, remote: Option<SocketAddr>, headers: HeaderMap, bytes: Bytes) -> Result<impl Reply, Infallible> {
    let reply = response(method, path, remote, headers, bytes, StatusCode::OK);
    Ok(reply)
}

async fn not_found(method: Method, path: FullPath, remote: Option<SocketAddr>, headers: HeaderMap, bytes: Bytes) -> Result<impl Reply, Infallible> {
    let reply = response(method, path, remote, headers, bytes, StatusCode::NOT_FOUND);
    Ok(reply)
}

fn response(method: Method, path: FullPath, remote: Option<SocketAddr>, headers: HeaderMap, bytes: Bytes, status: StatusCode) -> impl Reply {
    let metric_counter = metrics::ECHO_COUNT
        .get_metric_with_label_values(&[method.as_str()])
        .unwrap();
    let server = whoami::hostname();
    let result = api::EchoResponse::new(remote, method, headers, path, bytes, server);
    let response = warp::reply::json(&result);
    metric_counter.inc();
    warp::reply::with_status(response, status)
}

pub fn echo_handler() -> BoxedFilter<(impl warp::Reply,)> {
    warp::method()
        .and(warp::path::full())
        .and(warp::addr::remote())
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .and_then(ok)
        .boxed()
}

pub fn default_handler() -> BoxedFilter<(impl warp::Reply,)> {
    warp::method()
        .and(warp::path::full())
        .and(warp::addr::remote())
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .and_then(not_found)
        .boxed()
}

pub fn template_handler() -> BoxedFilter<(impl warp::Reply,)> {
    warp::get()
        .and(warp::addr::remote())
        .and(warp::header::headers_cloned())
        .map(|remote: Option<SocketAddr>, headers: HeaderMap| {
            let metric_counter = metrics::ECHO_COUNT
                .get_metric_with_label_values(&["GET"])
                .unwrap();
            metric_counter.inc();
            let server = whoami::hostname();
            let template = api::IndexTemplate::new(remote, headers, server).render().unwrap();
            warp::reply::html(template)
        }).boxed()
}

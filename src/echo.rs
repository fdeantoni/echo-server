use std::convert::Infallible;

use tracing::*;

use bytes::Bytes;
use warp::{Filter, Reply, filters::BoxedFilter, hyper::{HeaderMap, Method, StatusCode}, path::FullPath};

use askama::Template;

use crate::{api, metrics};

#[tracing::instrument]
async fn ok(method: Method, path: FullPath, headers: HeaderMap, bytes: Bytes) -> Result<impl Reply, Infallible> {
    let reply = response(method, path, headers, bytes, StatusCode::OK);
    Ok(reply)
}

#[tracing::instrument]
async fn not_found(method: Method, path: FullPath, headers: HeaderMap, bytes: Bytes) -> Result<impl Reply, Infallible> {
    let reply = response(method, path, headers, bytes, StatusCode::NOT_FOUND);
    Ok(reply)
}

#[tracing::instrument]
fn response(method: Method, path: FullPath, headers: HeaderMap, bytes: Bytes, status: StatusCode) -> impl Reply {
    let metric_counter = metrics::ECHO_COUNT
        .get_metric_with_label_values(&[method.as_str()])
        .unwrap();
    let server = whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string());
    let result = api::EchoResponse::new(method, headers, path, bytes, server);
    let response = warp::reply::json(&result);
    info!(monotonic_counter.echo = 1, "handled request");
    metric_counter.inc();
    warp::reply::with_status(response, status)
}

pub fn echo_handler() -> BoxedFilter<(impl warp::Reply,)> {
    warp::method()
        .and(warp::path::full())
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .and_then(ok)
        .boxed()
}

pub fn default_handler() -> BoxedFilter<(impl warp::Reply,)> {
    warp::method()
        .and(warp::path::full())
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .and_then(not_found)
        .boxed()
}

#[tracing::instrument]
pub fn template_handler() -> BoxedFilter<(impl warp::Reply,)> {
    warp::get()
        .and(warp::header::headers_cloned())
        .map(|headers: HeaderMap| {
            let metric_counter = metrics::ECHO_COUNT
                .get_metric_with_label_values(&["GET"])
                .unwrap();
            metric_counter.inc();
            info!(monotonic_counter.echo = 1);
            let server = whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string());
            let template = api::IndexTemplate::new(headers, server).render().unwrap();
            warp::reply::html(template)
        }).boxed()
}

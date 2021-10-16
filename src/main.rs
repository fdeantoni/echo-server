#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;

mod api;
mod ws;
mod sse;
mod metrics;

use std::net::SocketAddr;
use std::str::FromStr;

use tokio::signal::unix::{signal, SignalKind};
use tokio::time::Duration;
use warp::*;

use tiny_tokio_actor::*;
use warp::hyper::{HeaderMap, Method, StatusCode};
use warp::path::FullPath;


#[derive(Clone, Debug)]
pub struct ServerEvent(String);

// Mark the struct as a system event message.
impl SystemEvent for ServerEvent {}

#[tokio::main]
async fn main() {
    let path = std::path::Path::new(".env");
    dotenv::from_path(path).ok();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info,tiny_tokio_actor=debug,websocket=debug");
    }
    env_logger::init();

    let addr = std::env::var("HOST_PORT")
        .ok()
        .and_then(|string| SocketAddr::from_str(&string).ok())
        .unwrap_or_else(|| SocketAddr::from_str("127.0.0.1:9000").unwrap());

    // Create the event bus and actor system
    let bus = EventBus::<ServerEvent>::new(1000);
    let system = ActorSystem::new("echo", bus);

    // prometheus metrics
    let metrics = metrics::metrics_handler();

    // Create the warp WebSocket route
    let ws = warp::path!("ws")
        .and(warp::any().map(move || system.clone()))
        .and(warp::addr::remote())
        .and(warp::ws())
        .map(|system: ActorSystem<ServerEvent>, remote: Option<SocketAddr>, ws: warp::ws::Ws| {
            ws.on_upgrade(move |websocket| ws::start_ws(system, remote, websocket) )
        });

    let sse = warp::path("sse")
        .and(warp::get())
        .and_then(sse::sse_stream);

    // The default route that accepts anything
    let echo = warp::path("echo")
        .and(warp::method())
        .and(warp::path::full())
        .and(warp::addr::remote())
        .and(warp::header::headers_cloned())
        .map(|method: Method, path: FullPath, remote: Option<SocketAddr>, headers: HeaderMap| {
            let metric_counter = metrics::ECHO_COUNT
                .get_metric_with_label_values(&[method.as_str()])
                .unwrap();
            let result = api::Response::new(remote, headers, path);
            let response = warp::reply::json(&result);
            metric_counter.inc();
            Ok(warp::reply::with_status(response, StatusCode::OK))
        });

    let index = warp::path::end()
        .and(warp::get())
        .and(warp::addr::remote())
        .and(warp::header::headers_cloned())
        .map(|remote: Option<SocketAddr>, headers: HeaderMap| {
            let metric_counter = metrics::ECHO_COUNT
                .get_metric_with_label_values(&["GET"])
                .unwrap();
            let result = format!("<p>Source: {:?}</p><p>Headers: {:?}</p>", remote, headers);
            let response = warp::reply::html(result);
            metric_counter.inc();
            response
        });

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE"])
        .allow_headers(vec!["Content-Type"]);

    // Create the warp routes
    let routes = index
        .or(metrics)
        .or(ws)
        .or(sse)
        .or(echo)
        .with(cors)
        .with(warp::log("echo-server"));

    // Start the server
    let (server_shutdown_tx, server_shutdown_rx) = tokio::sync::oneshot::channel();
    let (_, server) = warp::serve(routes)
        .bind_with_graceful_shutdown(addr, async {
            server_shutdown_rx.await.ok();
        });
    tokio::task::spawn(server);
    ::log::info!("Echo server running on {}", &addr);

    let mut signal_stream = signal(SignalKind::interrupt()).unwrap();
    signal_stream.recv().await;

    let _ = server_shutdown_tx.send(());
    tokio::time::sleep(Duration::from_millis(200)).await;
}

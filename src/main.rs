#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;

mod api;
mod echo;
mod ws;
mod sse;
mod metrics;

use std::net::SocketAddr;
use std::str::FromStr;

use tokio::signal::unix::{signal, SignalKind};
use tokio::time::Duration;
use warp::*;

use tiny_tokio_actor::*;


#[derive(Clone, Debug)]
pub struct ServerEvent(String);

// Mark the struct as a system event message.
impl SystemEvent for ServerEvent {}

#[tokio::main]
async fn main() {
    let path = std::path::Path::new(".env");
    dotenv::from_path(path).ok();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
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
    let ws_route = warp::path!("ws")
        .and(warp::any().map(move || system.clone()))
        .and(warp::addr::remote())
        .and(warp::ws())
        .map(|system: ActorSystem<ServerEvent>, remote: Option<SocketAddr>, ws: warp::ws::Ws| {
            ws.on_upgrade(move |websocket| ws::start_ws(system, remote, websocket) )
        }).boxed();

    let sse_route = warp::path("sse")
        .and(warp::get())
        .and_then(sse::sse_stream)
        .boxed();

    // The default route that accepts anything
    let echo_route = warp::path("echo").and(echo::echo_handler());

    let index_route = warp::path::end()
        .and(echo::template_handler());

    let default_route = warp::any().and(echo::default_handler());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE"])
        .allow_headers(vec!["Content-Type"]);

    // Create the warp routes
    let routes = index_route
        .or(metrics)
        .or(ws_route)
        .or(sse_route)
        .or(echo_route)
        .or(default_route)
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

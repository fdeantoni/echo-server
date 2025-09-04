#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;

mod api;
mod echo;
mod ws;
mod sse;
mod metrics;
mod expensive;

use std::net::SocketAddr;
use std::str::FromStr;

use tracing::*;
use otlp_logger::OtlpLogger;
use warp::*;

use tiny_tokio_actor::*;


#[derive(Clone, Debug)]
pub struct ServerEvent;

// Mark the struct as a system event message.
impl SystemEvent for ServerEvent {}

#[tokio::main]
async fn main() {
    let path = std::path::Path::new(".env");
    dotenvy::from_path(path).ok();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    let logger: OtlpLogger = otlp_logger::init().await.expect("Initialized logger");

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
        .and(warp::ws())
        .map(|system: ActorSystem<ServerEvent>, ws: warp::ws::Ws| {
            ws.on_upgrade(move |websocket| ws::start_ws(system, websocket) )
        }).boxed();

    let sse_route = warp::path("sse")
        .and(warp::get())
        .and_then(sse::sse_stream)
        .boxed();

    let index_route = warp::path::end()
        .and(echo::template_handler());

    let echo_route = warp::path("echo").and(echo::echo_handler());

    let expensive_route = warp::path("expensive").and(expensive::expensive_handler());

    let default_route = warp::any().and(echo::default_handler());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE"])
        .allow_headers(vec!["Content-Type"]);

    let log = warp::log::custom(|info| {
        info!(
            target: "echo-server",
            "\"{} {}\" {} \"{}\" \"{}\" {:?}",
            info.method(),
            info.path(),
            info.status().as_u16(),
            info.referer().unwrap_or("-"),
            info.user_agent().unwrap_or("-"),
            info.elapsed()
        );        
    });

    // Create the warp routes
    let routes = index_route
        .or(expensive_route)
        .or(metrics)
        .or(ws_route)
        .or(sse_route)
        .or(echo_route)
        .or(default_route)
        
        .with(cors)
        .with(log);

    // Start the server
    info!(%addr, "Echo server running");
    warp::serve(routes).run(addr).await;

    logger.shutdown();

}

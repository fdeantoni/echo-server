mod api;
mod ws;
mod sse;

use std::net::SocketAddr;
use std::str::FromStr;

use warp::*;

use tiny_tokio_actor::*;
use warp::hyper::{HeaderMap, StatusCode};


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


    // Create the warp WebSocket route
    let ws = warp::path!(".ws")
        .and(warp::any().map(move || system.clone()))
        .and(warp::addr::remote())
        .and(warp::ws())
        .map(|system: ActorSystem<ServerEvent>, remote: Option<SocketAddr>, ws: warp::ws::Ws| {
            ws.on_upgrade(move |websocket| ws::start_ws(system, remote, websocket) )
        });

    let sse = warp::path(".sse")
        .and(warp::get())
        .and_then(sse::sse_stream);

    // The default route that accepts anything
    let default = warp::any()
        .and(warp::addr::remote())
        .and(warp::header::headers_cloned())
        .map(|remote: Option<SocketAddr>, headers: HeaderMap| {
            let result = api::Response::new(remote, headers);
            let response = warp::reply::json(&result);
            Ok(warp::reply::with_status(response, StatusCode::OK))
        });

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE"])
        .allow_headers(vec!["Content-Type"]);

    // Create the warp routes
    let routes = ws
        .or(sse)
        .or(default)
        .with(cors)
        .with(warp::log("echo-server"));

    // Start the server
    warp::serve(routes).run(addr).await;
}

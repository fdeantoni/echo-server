mod ws;

use std::net::SocketAddr;
use std::str::FromStr;

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
        std::env::set_var("RUST_LOG", "info,tiny_tokio_actor=debug,websocket=debug");
    }
    env_logger::init();

    let addr = std::env::var("HOST_PORT")
        .ok()
        .and_then(|string| SocketAddr::from_str(&string).ok())
        .unwrap_or_else(|| SocketAddr::from_str("127.0.0.1:9000").unwrap());

    // Create the event bus and actor system
    let bus = EventBus::<ServerEvent>::new(1000);
    let system = ActorSystem::new("test", bus);

    // Create the warp WebSocket route
    let ws = warp::path!("echo")
        .and(warp::any().map(move || system.clone()))
        .and(warp::addr::remote())
        .and(warp::ws())
        .map(|system: ActorSystem<ServerEvent>, remote: Option<SocketAddr>, ws: warp::ws::Ws| {
            ws.on_upgrade(move |websocket| ws::start_ws(system, remote, websocket) )
        });

    // Create the warp routes (websocket only in this case, with warp logging added)
    let routes = ws.with(warp::log("echo-server"));

    // Start the server
    warp::serve(routes).run(addr).await;
}

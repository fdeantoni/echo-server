use futures::StreamExt;

use tokio::sync::mpsc;
use tokio::task;
use tokio_stream::wrappers::UnboundedReceiverStream;

use uuid::Uuid;
use warp::ws::WebSocket;

use tracing::*;

use tiny_tokio_actor::*;

use crate::ServerEvent;

#[derive(Clone)]
struct WsActor {
    sender: mpsc::UnboundedSender<warp::ws::Message>
}

impl WsActor {
    pub fn new(sender: mpsc::UnboundedSender<warp::ws::Message>) -> Self {
        WsActor {
            sender
        }
    }
}

impl Actor<ServerEvent> for WsActor {}

#[derive(Clone, Debug)]
struct EchoRequest(warp::ws::Message);

impl Message for EchoRequest {
    type Response = ();
}

#[async_trait]
impl Handler<ServerEvent, EchoRequest> for WsActor {
    async fn handle(&mut self, msg: EchoRequest, _ctx: &mut ActorContext<ServerEvent>) {
        info!(?msg, "websocket received message");
        self.sender.send(msg.0).unwrap()
    }
}

// Starts a new echo actor on our actor system
pub async fn start_ws(system: ActorSystem<ServerEvent>, websocket: WebSocket) {

    // Split out the websocket into incoming and outgoing
    let (ws_out, mut ws_in) = websocket.split();

    // Create an unbounded channel where the actor can send its responses to ws_out
    let (sender, receiver) = mpsc::unbounded_channel();
    let receiver = UnboundedReceiverStream::new(receiver);
    task::spawn(receiver.map(Ok).forward(ws_out));

    // Create a new echo actor with the newly created sender
    let actor = WsActor::new(sender);
    // Use the websocket client address to generate a unique actor name
    let actor_name = format!("echo-actor-{}", &Uuid::new_v4());
    // Launch the actor on our actor system
    let actor_ref = system.create_actor(&actor_name, actor).await.unwrap();

    // Loop over all websocket messages received over ws_in
    while let Some(result) = ws_in.next().await {
        // If no error, we tell the websocket message to the echo actor, otherwise break the loop
        match result {
            Ok(msg) => actor_ref.tell(EchoRequest(msg)).unwrap(),
            Err(error) => {
                error!(?error, "error processing ws message");
                break;
            }
        };
    }

    // The loop has been broken, kill the echo actor
    system.stop_actor(actor_ref.path()).await;
}
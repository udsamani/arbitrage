use common::{Context, SpawnResult, Worker};
use models::InternalMessage;
use tokio::sync::broadcast::Sender;
use warp::{ws::WebSocket, Filter};

#[derive(Clone)]
pub struct Endpoint {
    context: Context,
    broadcaster: Sender<InternalMessage>,
    port: u16,
}


#[allow(unused)]
impl Endpoint {
    pub fn new(context: Context, broadcaster: Sender<InternalMessage>) -> Self {
        let port = context.config.get_int("websocket_server_endpoint").unwrap_or(9027) as u16;
        Self { context, broadcaster, port }
    }
}


impl Worker for Endpoint {
    fn spawn(&mut self) -> SpawnResult {

        let endpoint = self.clone();
        tokio::spawn(async move {
            let receiver = endpoint.broadcaster.clone();
            let receiver = warp::any().map(move || receiver.clone());

            let stream_v1 = warp::path!("stream" / "v1")
                .and(warp::ws())
                .and(receiver)
                .map(|ws: warp::ws::Ws, receiver| {
                    ws.on_upgrade(move |socket| socket_connected(socket, receiver))
                });

            let not_found = warp::path::end()
                .map(|| {
                    warp::reply::with_status(
                        warp::reply::json(&"not found"),
                        warp::http::StatusCode::NOT_FOUND
                    )
                });

            let routes = stream_v1.or(not_found);

            let mut app = endpoint.context.app.subscribe();
            let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(
                ([0, 0, 0, 0], endpoint.port),
                async move {
                    app.recv().await.ok();
                }
            );

            server.await;

            Ok("websocket server exited".to_string())
        })
    }
}

async fn socket_connected(ws: WebSocket, broadcaster: Sender<InternalMessage>) {
    let mut socket = super::websocket::WebSocket::new(broadcaster.subscribe());
    tokio::spawn(async move {
        let result = socket.serve(ws).await;
        match result {
            Ok(_) => {
                log::info!("websocket connection closed normally");
            }
            Err(e) => {
                log::error!("error serving websocket: {}", e);
            }
        }
    });
}

use std::net::SocketAddr;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use rpi::FingerForceData;
use tokio::net::ToSocketAddrs;
use tower_http::cors::CorsLayer;
use zenoh::prelude::r#async::*;

#[tokio::main]
async fn main() {
    server("0.0.0.0:8080").await;
}

trait ValueToMsg {
    fn key() -> String;
    fn value_to_msg(s: Sample) -> Message;
}

struct Left;
struct Right;

struct FingerCamera<L> {
    _p: std::marker::PhantomData<L>,
}

impl ValueToMsg for FingerCamera<Left> {
    fn key() -> String {
        "finger/left/image".into()
    }

    fn value_to_msg(s: Sample) -> Message {
        let data: Vec<u8> = s.value.try_into().unwrap();
        Message::Binary(data)
    }
}

impl ValueToMsg for FingerCamera<Right> {
    fn key() -> String {
        "finger/right/image".into()
    }

    fn value_to_msg(s: Sample) -> Message {
        let data: Vec<u8> = s.value.try_into().unwrap();
        Message::Binary(data)
    }
}

struct FingerForce<L> {
    _p: std::marker::PhantomData<L>,
}

impl ValueToMsg for FingerForce<Left> {
    fn key() -> String {
        "finger/left/force".into()
    }

    fn value_to_msg(s: Sample) -> Message {
        let v = s.value.try_into().unwrap();
        let data = serde_json::from_value::<FingerForceData>(v).unwrap();
        Message::Text(serde_json::to_string(&data).unwrap())
    }
}

impl ValueToMsg for FingerForce<Right> {
    fn key() -> String {
        "finger/right/force".into()
    }

    fn value_to_msg(s: Sample) -> Message {
        let v = s.value.try_into().unwrap();
        let data = serde_json::from_value::<FingerForceData>(v).unwrap();
        Message::Text(serde_json::to_string(&data).unwrap())
    }
}

async fn server<A: ToSocketAddrs>(addr: A) {
    let app = Router::new()
        .route("/ping", get(ping))
        .route(
            "/left_finger/image",
            get(streaming_handler::<FingerCamera<Left>>),
        )
        .route(
            "/right_finger/image",
            get(streaming_handler::<FingerCamera<Right>>),
        )
        .route(
            "/left_finger/force",
            get(streaming_handler::<FingerForce<Left>>),
        )
        .route(
            "/right_finger/force",
            get(streaming_handler::<FingerForce<Right>>),
        )
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn ping() -> impl IntoResponse {
    StatusCode::OK
}

async fn streaming_handler<V: ValueToMsg + 'static>(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_streaming_socket::<V>(socket))
}

async fn handle_streaming_socket<V: ValueToMsg + 'static>(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();
    let mut send_task = tokio::spawn(async move {
        let session = zenoh::open(config::default()).res().await.unwrap();
        let data_subscriber = session.declare_subscriber(V::key()).res().await.unwrap();
        while let Ok(s) = data_subscriber.recv_async().await {
            // let value = s.value;
            let msg = V::value_to_msg(s);
            // let data = serde_json::from_value::<AngleData>(json_value).unwrap();
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut recv_close = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        _ = (&mut send_task) => {
            recv_close.abort();
        },
        _ = (&mut recv_close) => {
            send_task.abort();
        }
    }
}

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::CloseFrame;

//allows to split the websocket stream into separate TX and RX branches
use futures::{sink::SinkExt, stream::StreamExt};

use crate::imu::IMUData;

#[derive(Debug, Serialize)]
enum Data {
    IMU(IMUData),
    Finger
}

struct State {
    imu: broadcast::Receiver<IMUData>
}

async fn web_serve() {
    let app = Router::new()
        .route("/streaming", get(ws_streaming_handler))
        .route("/data", get(ws_data_handler))
        // .route(path, method_router)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(CorsLayer::new().allow_origin(Any));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn ws_streaming_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_streaming_socket(socket, addr))
}

async fn ws_data_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_data_socket(socket, addr))
}

async fn handle_streaming_socket(mut socket: WebSocket, who: SocketAddr) {
    // socket.send(Message::Binary());
}

async fn handle_data_socket(mut socket: WebSocket, who: SocketAddr) {
    // socket.send(Message::Text(
    //     serde_json::to_string(Data::IMU(IMUData))
    // ));
}

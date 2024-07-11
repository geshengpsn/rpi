use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures::{SinkExt, StreamExt};
use rpi::{FingerForceResult, IMUData, RecordCommand};
use tokio::net::ToSocketAddrs;
use zenoh::prelude::r#async::*;

use tower_http::cors::CorsLayer;

struct AppState {
    imu_owner: tokio::sync::Mutex<Option<SocketAddr>>,
    camera_owner: tokio::sync::Mutex<Option<SocketAddr>>,
    left_finger_owner: tokio::sync::Mutex<Option<SocketAddr>>,
    right_finger_owner: tokio::sync::Mutex<Option<SocketAddr>>,
}

#[tokio::main]
async fn main() {
    server("0.0.0.0:8080").await;
}

async fn server<A: ToSocketAddrs>(addr: A) {
    let state = AppState {
        imu_owner: tokio::sync::Mutex::new(None),
        camera_owner: tokio::sync::Mutex::new(None),
        left_finger_owner: tokio::sync::Mutex::new(None),
        right_finger_owner: tokio::sync::Mutex::new(None),
    };

    let app = Router::new()
        .route("/imu", get(imu_streaming_handler))
        .route("/imu/calibrate", post(imu_calibrate_handler))
        .route("/camera", get(camera_streaming_handler))
        .route(
            "/left_finger/force",
            get(left_finger_force_streaming_handler),
        )
        .route(
            "/right_finger/force",
            get(right_finger_force_streaming_handler),
        )
        .route("/recordstart", post(record_start_handler))
        .route("/recordend", post(record_end_handler))
        .layer(CorsLayer::permissive())
        .with_state(state.into());

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn record_start_handler(Json(cmd): Json<RecordCommand>) -> StatusCode {
    let session = zenoh::open(config::default()).res().await.unwrap();
    let cmd_pub = session.declare_publisher("cmd/record").res().await.unwrap();
    cmd_pub
        .put(serde_json::to_value(cmd).unwrap())
        .res()
        .await
        .unwrap();
    StatusCode::OK
}

async fn record_end_handler() -> StatusCode {
    let session = zenoh::open(config::default()).res().await.unwrap();
    let cmd_pub = session.declare_publisher("cmd/record").res().await.unwrap();
    cmd_pub
        .put(serde_json::to_value(RecordCommand::End).unwrap())
        .res()
        .await
        .unwrap();
    StatusCode::OK
}

async fn imu_streaming_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_imu_streaming_socket(socket, state, addr))
}

async fn handle_imu_streaming_socket(socket: WebSocket, state: Arc<AppState>, addr: SocketAddr) {
    let (mut sender, mut receiver) = socket.split();
    {
        let mut owner = state.imu_owner.lock().await;
        match owner.as_ref() {
            Some(current_addr) => {
                if addr != *current_addr {
                    return;
                }
            }
            None => {
                // new owner
                owner.replace(addr);
            }
        }
    }

    let send_state = state.clone();
    let mut send_task = tokio::spawn(async move {
        let session = zenoh::open(config::default()).res().await.unwrap();
        let data_subscriber = session.declare_subscriber("imu/data").res().await.unwrap();
        while let Ok(s) = data_subscriber.recv_async().await {
            let json_value = s.value.try_into().unwrap();
            let data = serde_json::from_value::<IMUData>(json_value).unwrap();
            sender
                .send(Message::Text(serde_json::to_string(&data).unwrap()))
                .await
                .unwrap();
        }
        send_state.imu_owner.lock().await.take();
    });

    let recv_state = state.clone();
    let mut recv_close = tokio::spawn(async move {
        // let session = zenoh::open(config::default()).res().await.unwrap();
        // let data_subscriber = session.declare_subscriber("imu/data").res().await.unwrap();
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
        recv_state.imu_owner.lock().await.take();
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
    // end_state.owner.lock().unwrap().take();
}

async fn imu_calibrate_handler() -> StatusCode {
    let session = zenoh::open(config::default()).res().await.unwrap();
    let cmd_pub = session.declare_publisher("imu/cmd").res().await.unwrap();
    cmd_pub.put(0u8).res().await.unwrap();
    StatusCode::OK
}

async fn camera_streaming_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_camera_streaming_socket(socket, state, addr))
}

async fn handle_camera_streaming_socket(socket: WebSocket, state: Arc<AppState>, addr: SocketAddr) {
    let (mut sender, mut receiver) = socket.split();
    {
        let mut owner = state.camera_owner.lock().await;
        match owner.as_ref() {
            Some(current_addr) => {
                if addr != *current_addr {
                    return;
                }
            }
            None => {
                // new owner
                owner.replace(addr);
            }
        }
    }
    let send_state = state.clone();
    let mut send_task = tokio::spawn(async move {
        let session = zenoh::open(config::default()).res().await.unwrap();
        let data_subscriber = session.declare_subscriber("camera").res().await.unwrap();
        while let Ok(s) = data_subscriber.recv_async().await {
            let data: Vec<u8> = s.value.try_into().unwrap();
            sender.send(Message::Binary(data)).await.unwrap();
        }
        send_state.camera_owner.lock().await.take();
    });

    let recv_state = state.clone();
    let mut recv_close = tokio::spawn(async move {
        // let session = zenoh::open(config::default()).res().await.unwrap();
        // let data_subscriber = session.declare_subscriber("imu/data").res().await.unwrap();
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
        recv_state.camera_owner.lock().await.take();
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

async fn left_finger_force_streaming_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_left_finger_force_streaming_socket(socket, state, addr))
}

async fn handle_left_finger_force_streaming_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    addr: SocketAddr,
) {
    let (mut sender, mut receiver) = socket.split();
    {
        let mut owner = state.left_finger_owner.lock().await;
        match owner.as_ref() {
            Some(current_addr) => {
                if addr != *current_addr {
                    return;
                }
            }
            None => {
                // new owner
                owner.replace(addr);
            }
        }
    }
    let send_state = state.clone();
    let mut send_task = tokio::spawn(async move {
        let session = zenoh::open(config::default()).res().await.unwrap();
        let data_subscriber = session
            .declare_subscriber("left_finger/force")
            .res()
            .await
            .unwrap();
        while let Ok(s) = data_subscriber.recv_async().await {
            let json_value = s.value.try_into().unwrap();
            let data = serde_json::from_value::<FingerForceResult>(json_value).unwrap();
            sender
                .send(Message::Text(serde_json::to_string(&data).unwrap()))
                .await
                .unwrap();
        }
        send_state.left_finger_owner.lock().await.take();
    });

    let recv_state = state.clone();
    let mut recv_close = tokio::spawn(async move {
        // let session = zenoh::open(config::default()).res().await.unwrap();
        // let data_subscriber = session.declare_subscriber("imu/data").res().await.unwrap();
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
        recv_state.left_finger_owner.lock().await.take();
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

async fn right_finger_force_streaming_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_right_finger_force_streaming_socket(socket, state, addr))
}

async fn handle_right_finger_force_streaming_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    addr: SocketAddr,
) {
    let (mut sender, mut receiver) = socket.split();
    {
        let mut owner = state.right_finger_owner.lock().await;
        match owner.as_ref() {
            Some(current_addr) => {
                if addr != *current_addr {
                    return;
                }
            }
            None => {
                // new owner
                owner.replace(addr);
            }
        }
    }
    let send_state = state.clone();
    let mut send_task = tokio::spawn(async move {
        let session = zenoh::open(config::default()).res().await.unwrap();
        let data_subscriber = session
            .declare_subscriber("right_finger/force")
            .res()
            .await
            .unwrap();
        while let Ok(s) = data_subscriber.recv_async().await {
            let json_value = s.value.try_into().unwrap();
            let data = serde_json::from_value::<FingerForceResult>(json_value).unwrap();
            sender
                .send(Message::Text(serde_json::to_string(&data).unwrap()))
                .await
                .unwrap();
        }
        send_state.right_finger_owner.lock().await.take();
    });

    let recv_state = state.clone();
    let mut recv_close = tokio::spawn(async move {
        // let session = zenoh::open(config::default()).res().await.unwrap();
        // let data_subscriber = session.declare_subscriber("imu/data").res().await.unwrap();
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
        recv_state.right_finger_owner.lock().await.take();
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

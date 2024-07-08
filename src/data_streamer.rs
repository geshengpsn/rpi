use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use crossbeam::channel::{unbounded, Receiver, Sender, TrySendError};
use opencv::{
    core::{Mat, Vector, VectorToVec},
    imgcodecs::imencode_def,
};
use tokio::net::ToSocketAddrs;

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread::spawn,
};

use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;

use crate::data_saver::FrameData;

pub fn spawn_video_streaming<A: ToSocketAddrs>(rx: Receiver<Mat>, tx: Sender<Mat>, addr: A, path: &str) {
    let (web_tx, web_rx) = unbounded();
    spawn(move || {
        while let Ok(data) = rx.recv() {
            let data_copy = data.clone();
            if let Err(TrySendError::Disconnected(_)) = web_tx.try_send(data) {
                panic!("web_tx disconnected")
            }
            tx.send(data_copy).unwrap();
        }
    });
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(serve_video_streaming(web_rx, addr, path));
}

struct VideoAppState {
    rx: Receiver<Mat>,
    owner: Mutex<Option<SocketAddr>>,
}

async fn serve_video_streaming<A: ToSocketAddrs>(rx: Receiver<Mat>, addr: A, path: &str) {
    let state = Arc::new(VideoAppState {
        rx,
        owner: Mutex::new(None),
    });
    let app = Router::new()
        .route(path, get(ws_video_streaming_handler))
        // .route(path, method_router)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(CorsLayer::new().allow_origin(Any))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn ws_video_streaming_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<VideoAppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_video_streaming_socket(socket, state, addr))
}

async fn handle_video_streaming_socket(
    mut socket: WebSocket,
    state: Arc<VideoAppState>,
    addr: SocketAddr,
) {
    if state.owner.lock().unwrap().is_some() {
        socket
            .send(Message::Text(String::from("Username already taken.")))
            .await
            .unwrap();
    } else {
        {
            state.owner.lock().unwrap().replace(addr);
        }
        while let Ok(data) = state.rx.recv() {
            let mut v = Vector::<u8>::new();
            imencode_def(".jpg", &data, &mut v).unwrap();
            let _ = socket.send(Message::Binary(v.to_vec())).await;
        }
    }
}

pub fn spawn_data_streaming<D: FrameData, A: ToSocketAddrs>(rx: Receiver<D>, tx: Sender<D>, addr: A, path: &str) {
    let (web_tx, web_rx) = unbounded();
    spawn(move || {
        while let Ok(data) = rx.recv() {
            let data_copy = data.clone();
            if let Err(TrySendError::Disconnected(_)) = web_tx.try_send(data) {
                panic!("web_tx disconnected")
            }
            tx.send(data_copy).unwrap();
        }
    });
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(serve_data_streaming(web_rx, addr, path));
}

struct AppState<D: FrameData> {
    rx: Receiver<D>,
    owner: Mutex<Option<SocketAddr>>,
}

async fn serve_data_streaming<D: FrameData, A: ToSocketAddrs>(rx: Receiver<D>, addr: A, path: &str) {
    let state = Arc::new(AppState {
        rx,
        owner: Mutex::new(None),
    });
    let app = Router::new()
        .route(path, get(ws_streaming_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(CorsLayer::new().allow_origin(Any))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn ws_streaming_handler<D: FrameData>(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState<D>>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_streaming_socket(socket, state, addr))
}

async fn handle_streaming_socket<D: FrameData>(
    mut socket: WebSocket,
    state: Arc<AppState<D>>,
    addr: SocketAddr,
) {
    if state.owner.lock().unwrap().is_some() {
        socket
            .send(Message::Text(String::from("Username already taken.")))
            .await
            .unwrap();
    } else {
        {
            state.owner.lock().unwrap().replace(addr);
        }
        while let Ok(data) = state.rx.recv() {
            let _ = socket
                .send(Message::Text(serde_json::to_string(&data).unwrap()))
                .await;
        }
    }
}

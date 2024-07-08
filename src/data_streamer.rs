use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use crossbeam::channel::Receiver;
use futures::{SinkExt, StreamExt};
use opencv::{
    core::{Mat, Vector, VectorToVec},
    imgcodecs::imencode_def,
};
use tokio::net::ToSocketAddrs;

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread::{spawn, JoinHandle},
    time::Duration,
};

use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;

use crate::data_saver::FrameData;

pub fn spawn_video_streaming<A: ToSocketAddrs + Send + 'static>(
    rx: Receiver<(Mat, Duration)>,
    addr: A,
    path: String,
) -> JoinHandle<()> {
    spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(serve_video_streaming(rx, addr, path.as_str()))
    })
}

struct VideoAppState {
    rx: Receiver<(Mat, Duration)>,
    owner: Mutex<Option<SocketAddr>>,
}

async fn serve_video_streaming<A: ToSocketAddrs>(
    rx: Receiver<(Mat, Duration)>,
    addr: A,
    path: &str,
) {
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

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
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
    socket: WebSocket,
    state: Arc<VideoAppState>,
    addr: SocketAddr,
) {
    let end_state = state.clone();
    {
        let mut owner = state.owner.lock().unwrap();
        match owner.as_ref() {
            Some(current_addr) => {
                if addr != *current_addr {
                    return;
                }
            }
            None => {
                // new owner
                println!("new: {addr}");
                owner.replace(addr);
            }
        }
    }
    let (mut sender, mut receiver) = socket.split();
    // Spawn a task that will push several messages to the client (does not matter what client does)
    let mut send_task = tokio::spawn(async move {
        while let Ok((data, duration)) = state.rx.recv() {
            let mut v = Vector::<u8>::new();
            imencode_def(".jpg", &data, &mut v).unwrap();
            let _ = sender.send(Message::Binary(v.to_vec())).await;
            let _ = sender
                .send(Message::Text(serde_json::to_string(&duration).unwrap()))
                .await;
        }
    });

    // This second task will receive messages from client and print them on server console
    let mut recv_close = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => {
                    println!("close: {addr}");
                    end_state.owner.lock().unwrap().take();
                    break;
                }
                m => {
                    println!("{m:?}");
                }
            }
            // if let Message::Close(_) = msg {
            //     println!("close: {addr}");
            //     end_state.owner.lock().unwrap().take();
            //     break;
            // }
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

pub fn spawn_data_streaming<D: FrameData, A: ToSocketAddrs + Send + 'static>(
    rx: Receiver<D>,
    addr: A,
    path: String,
) -> JoinHandle<()> {
    spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(serve_data_streaming(rx, addr, path.as_str()))
    })
}

struct AppState<D: FrameData> {
    rx: Receiver<D>,
    owner: Mutex<Option<SocketAddr>>,
}

async fn serve_data_streaming<D: FrameData, A: ToSocketAddrs>(
    rx: Receiver<D>,
    addr: A,
    path: &str,
) {
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

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
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
    socket: WebSocket,
    state: Arc<AppState<D>>,
    addr: SocketAddr,
) {
    let end_state = state.clone();
    {
        let mut owner = state.owner.lock().unwrap();
        match owner.as_ref() {
            Some(current_addr) => {
                if addr != *current_addr {
                    return;
                }
            }
            None => {
                // new owner
                println!("new: {addr}");
                owner.replace(addr);
            }
        }
    }
    let (mut sender, mut receiver) = socket.split();
    // Spawn a task that will push several messages to the client (does not matter what client does)
    let mut send_task = tokio::spawn(async move {
        while let Ok(data) = state.rx.recv() {
            let _ = sender
                .send(Message::Text(serde_json::to_string(&data).unwrap()))
                .await;
        }
    });

    // This second task will receive messages from client and print them on server console
    let mut recv_close = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                end_state.owner.lock().unwrap().take();
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

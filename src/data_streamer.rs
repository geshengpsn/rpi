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
use tokio::{net::ToSocketAddrs, sync::{broadcast::Receiver as AsyncReceiver, mpsc}};
use tracing::{info, instrument};

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

struct AsyncVideoAppState {
    rx: tokio::sync::Mutex<mpsc::Receiver<(Mat, Duration)>>,
    owner: tokio::sync::Mutex<Option<SocketAddr>>,
}


pub async fn video_stream<A: ToSocketAddrs>(
    rx: mpsc::Receiver<(Mat, Duration)>,
    addr: A,
    path: &str,
) {
    let state = Arc::new(AsyncVideoAppState {
        rx: tokio::sync::Mutex::new(rx),
        owner: tokio::sync::Mutex::new(None),
    });
    let app = Router::new()
        .route(path, get(ws_async_video_streaming_handler))
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


async fn ws_async_video_streaming_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AsyncVideoAppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_async_video_streaming_socket(socket, state, addr))
}

async fn handle_async_video_streaming_socket(
    socket: WebSocket,
    state: Arc<AsyncVideoAppState>,
    addr: SocketAddr,
) {
    let (mut sender, mut receiver) = socket.split();

    {
        let mut owner = state.owner.lock().await;
        match owner.as_ref() {
            Some(current_addr) => {
                if addr != *current_addr {
                    return;
                }
            }
            None => {
                // new owner
                info!("new: {addr}");
                owner.replace(addr);
            }
        }
    }

    let send_state = state.clone();
    let mut send_task = tokio::spawn(async move {
        let mut v = Vector::<u8>::new();
        // let a = send_state.rx.lock();
        while let Some((data, duration)) = send_state.rx.lock().await.recv().await {
            imencode_def(".jpg", &data, &mut v).unwrap();
            match sender.send(Message::Binary(v.to_vec())).await {
                Ok(_) => {}
                Err(e) => {
                    info!("ws send error: {e}");
                    break;
                }
            }

            match sender
                .send(Message::Text(serde_json::to_string(&duration).unwrap()))
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    info!("ws send error: {e}");
                    break;
                }
            }
        }
        send_state.owner.lock().await.take();
    });
    // send_task.abort()

    let recv_state = state.clone();
    let mut recv_close = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => {
                    info!("close: {addr}");
                    break;
                }
                m => {
                    info!("{m:?}");
                }
            }
        }
        recv_state.owner.lock().await.take();
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


struct AsyncRawVideoAppState {
    rx: tokio::sync::Mutex<mpsc::Receiver<(Vec<u8>, Duration)>>,
    owner: tokio::sync::Mutex<Option<SocketAddr>>,
}


pub async fn raw_video_stream<A: ToSocketAddrs>(
    rx: mpsc::Receiver<(Vec<u8>, Duration)>,
    addr: A,
    path: &str,
) {
    let state = Arc::new(AsyncRawVideoAppState {
        rx: tokio::sync::Mutex::new(rx),
        owner: tokio::sync::Mutex::new(None),
    });
    let app = Router::new()
        .route(path, get(ws_async_raw_video_streaming_handler))
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


async fn ws_async_raw_video_streaming_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AsyncRawVideoAppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_async_raw_video_streaming_socket(socket, state, addr))
}

async fn handle_async_raw_video_streaming_socket(
    socket: WebSocket,
    state: Arc<AsyncRawVideoAppState>,
    addr: SocketAddr,
) {
    let (mut sender, mut receiver) = socket.split();

    {
        let mut owner = state.owner.lock().await;
        match owner.as_ref() {
            Some(current_addr) => {
                if addr != *current_addr {
                    return;
                }
            }
            None => {
                // new owner
                info!("new: {addr}");
                owner.replace(addr);
            }
        }
    }

    let send_state = state.clone();
    let mut send_task = tokio::spawn(async move {
        // let a = send_state.rx.lock();
        while let Some((data, duration)) = send_state.rx.lock().await.recv().await {
            // imencode_def(".jpg", &data, &mut v).unwrap();
            match sender.send(Message::Binary(data)).await {
                Ok(_) => {}
                Err(e) => {
                    info!("ws send error: {e}");
                    break;
                }
            }

            match sender
                .send(Message::Text(serde_json::to_string(&duration).unwrap()))
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    info!("ws send error: {e}");
                    break;
                }
            }
        }
        send_state.owner.lock().await.take();
    });
    // send_task.abort()

    let recv_state = state.clone();
    let mut recv_close = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => {
                    info!("close: {addr}");
                    break;
                }
                m => {
                    info!("{m:?}");
                }
            }
        }
        recv_state.owner.lock().await.take();
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

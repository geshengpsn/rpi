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

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;

use crate::data_saver::FrameData;

pub fn spawn_data_streaming<D: FrameData>(rx: Receiver<D>) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(web_serve(rx));
}

struct AppState<D: FrameData> {
    rx: Receiver<D>,
    owner: Mutex<Option<SocketAddr>>,
}

async fn web_serve<D: FrameData>(rx: Receiver<D>) {
    let state = Arc::new(AppState {
        rx,
        owner: Mutex::new(None),
    });
    let app = Router::new()
        .route("/streaming", get(ws_streaming_handler))
        // .route(path, method_router)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(CorsLayer::new().allow_origin(Any))
        .with_state(state);

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
        state.owner.lock().unwrap().replace(addr);
        while let Ok(data) = state.rx.recv() {
            let _ = socket
                .send(Message::Text(serde_json::to_string(&data).unwrap()))
                .await;
        }
    }
}

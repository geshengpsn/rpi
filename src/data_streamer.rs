use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{stream::SplitSink, SinkExt, StreamExt};
use opencv::{
    core::{Mat, Vector, VectorToVec},
    imgcodecs::imencode_def,
};
use tokio::{net::ToSocketAddrs, sync::mpsc};
// use tracing::info;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};

use axum::extract::connect_info::ConnectInfo;

use crate::data_saver::FrameData;

impl<D: FrameData> WsSender for D {
    async fn ws_send(self, sender: &mut SplitSink<WebSocket, Message>) {
        sender
            .send(Message::Text(serde_json::to_string(&self).unwrap()))
            .await
            .unwrap();
    }
}

impl WsSender for (Mat, Duration) {
    async fn ws_send(self, sender: &mut SplitSink<WebSocket, Message>) {
        let mut v = Vector::<u8>::new();
        imencode_def(".jpg", &self.0, &mut v).unwrap();
        match sender.send(Message::Binary(v.to_vec())).await {
            Ok(_) => {}
            Err(e) => {
                // info!("ws send error: {e}");
                return;
            }
        }

        match sender
            .send(Message::Text(serde_json::to_string(&self.1).unwrap()))
            .await
        {
            Ok(_) => {}
            Err(e) => {
                // info!("ws send error: {e}");
            }
        }
    }
}

pub trait WsSender: Send + 'static {
    fn ws_send(
        self,
        sender: &mut SplitSink<WebSocket, Message>,
    ) -> impl std::future::Future<Output = ()> + Send;
}

pub struct NewAppState<S: WsSender> {
    rx: tokio::sync::Mutex<mpsc::Receiver<S>>,
    owner: tokio::sync::Mutex<Option<SocketAddr>>,
}

pub async fn stream_data<S: WsSender + Send, A: ToSocketAddrs>(
    rx: mpsc::Receiver<S>,
    addr: A,
    path: &str,
) {
    let state = NewAppState {
        rx: tokio::sync::Mutex::new(rx),
        owner: tokio::sync::Mutex::new(None),
    };
    let app = Router::new()
        .route(path, get(streaming_handler))
        // .route(path, method_router)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(CorsLayer::new().allow_origin(Any))
        .with_state(state.into());

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn streaming_handler<S: WsSender>(
    ws: WebSocketUpgrade,
    State(state): State<Arc<NewAppState<S>>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_streaming_socket(socket, state, addr))
}

async fn handle_streaming_socket<S: WsSender>(
    socket: WebSocket,
    state: Arc<NewAppState<S>>,
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
                // info!("new: {addr}");
                owner.replace(addr);
            }
        }
    }

    let send_state = state.clone();
    let mut send_task = tokio::spawn(async move {
        while let Some(s) = send_state.rx.lock().await.recv().await {
            s.ws_send(&mut sender).await;
        }
        send_state.owner.lock().await.take();
    });

    let recv_state = state.clone();
    let mut recv_close = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => {
                    // info!("close: {addr}");
                    break;
                }
                m => {
                    // info!("{m:?}");
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
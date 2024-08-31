use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::stream::SplitStream;
use futures_util::SinkExt;
use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{accept_async, WebSocketStream};

use std::net::SocketAddr;
use std::sync::Arc;

pub async fn handle_ws_server(
    raw_stream: TcpStream,
    addr: SocketAddr,
    shared_ws_conn_send_tx: Arc<Mutex<UnboundedSender<String>>>,
    shared_ws_conn_recv_rx: Arc<Mutex<UnboundedReceiver<String>>>,
) {
    let ws_stream = accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("WebSocket connection established: {}", addr);

    let (mut ws_conn_tx, ws_conn_rx) = ws_stream.split();

    let recv_task = tokio::spawn(handle_ws_server_rx(
        ws_conn_rx,
        Arc::clone(&shared_ws_conn_send_tx),
    ));

    while let Some(message) = shared_ws_conn_recv_rx.lock().await.next().await {
        if ws_conn_tx.send(message.into()).await.is_err() {
            info!("WebSocket connection closed: {}", addr);
            break;
        }
    }

    recv_task.await.unwrap();
}

async fn handle_ws_server_rx(
    mut ws_conn_rx: SplitStream<WebSocketStream<TcpStream>>,
    shared_ws_conn_send_tx: Arc<Mutex<UnboundedSender<String>>>,
) {
    while let Some(Ok(message)) = ws_conn_rx.next().await {
        shared_ws_conn_send_tx
            .lock()
            .await
            .start_send(message.to_string())
            .unwrap();
    }
}

use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};

use futures_util::StreamExt;

use crate::types::SharedWebsocketClient;
use tokio_tungstenite::tungstenite::protocol::Message;

pub async fn handle_websocket_client(
    ws_client: SharedWebsocketClient,
    ws_client_send_rx: UnboundedReceiver<String>,
    ws_client_recv_tx: UnboundedSender<String>,
) {
    tokio::select! {
        _ = handle_websocket_client_send(ws_client_send_rx, ws_client.clone()) => {},
        _ = handle_websocket_client_recv(ws_client_recv_tx, ws_client) => {},
    }
}

async fn handle_websocket_client_send(
    mut ws_client_send_rx: UnboundedReceiver<String>,
    ws_client: SharedWebsocketClient,
) {
    while let Some(message) = ws_client_send_rx.next().await {
        send_msg_ws_client(&ws_client, message).await;
    }
}

async fn handle_websocket_client_recv(
    mut ws_client_recv_tx: UnboundedSender<String>,
    ws_client: SharedWebsocketClient,
) {
    while let Some(Ok(message)) = recv_msg_ws_client(&ws_client).await {
        ws_client_recv_tx.start_send(message.to_string()).unwrap();
    }
}

async fn send_msg_ws_client(ws_client: &SharedWebsocketClient, message: String) {
    ws_client.lock().await.send_message(message).await;
}

async fn recv_msg_ws_client(
    ws_client: &SharedWebsocketClient,
) -> Option<Result<Message, tokio_tungstenite::tungstenite::Error>> {
    ws_client.lock().await.recv_next().await
}

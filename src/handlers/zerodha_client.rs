use super::client_helpers::{handle_binary_message, handle_zerodha_text_message};
use crate::types::{ConnectedClients, ZerodhaClientStream};
use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::protocol::Message;

#[instrument(skip(ws_client, connected_clients), fields(zerodha = true))]
pub async fn handle_zerodha_recv(
    mut ws_client: ZerodhaClientStream,
    connected_clients: ConnectedClients,
) {
    while let Some(Ok(msg)) = receive_next_message(&mut ws_client).await {
        match msg {
            Message::Binary(data) if data.len() > 2 => {
                handle_binary_message(&data, &connected_clients).await;
            }
            Message::Text(message) => {
                handle_zerodha_text_message(message).await;
            }
            _ => debug!("Message other than binary or text received."),
        }
    }
}

async fn receive_next_message(
    ws_client: &mut ZerodhaClientStream,
) -> Option<Result<Message, tokio_tungstenite::tungstenite::Error>> {
    ws_client.next().await
}

use super::dto::ZerodhaMessage;
use crate::types::ConnectedClients;
use serde_json::json;

pub async fn handle_binary_message(data: &[u8], connected_clients: &ConnectedClients) {
    let messages = ZerodhaMessage::parse(data);
    for message in messages {
        if let Some(clients) = connected_clients
            .lock()
            .await
            .get_mut(&message.instrument_token)
        {
            clients.retain_mut(|client_tx| {
                if !client_tx.is_closed() {
                    let message_json = serde_json::to_string(&message).unwrap();
                    if let Err(e) = client_tx.start_send_wrapped(message_json) {
                        warn!("[{}] :: Failed to send message: {:?}", client_tx.uuid, e);
                    }
                    true
                } else {
                    false
                }
            });
        }
    }
}

#[instrument(skip(message))]
pub async fn handle_zerodha_text_message(message: String) {
    if message != "\u{0000}" {
        let text_message_json = json!(message);
        info!("text recieve => {}", text_message_json);
    }
}

use super::dto::ZerodhaMessage;
use crate::types::{ConnectedClients, NODE_SERVICE_SENDER};
use serde_json::json;

fn send_to_node_service_idsender(message: String) {
    let mut node_service_id_sender = {
        let sender = NODE_SERVICE_SENDER.read().unwrap();
        sender.clone()
    };

    match node_service_id_sender {
        Some(ref mut id_sender) => {
            if let Err(e) = id_sender.start_send_wrapped(message.clone()) {
                warn!(
                    "[{}] :: Failed to send message to NodeJs Service: {:?}",
                    id_sender.uuid, e
                );

                let mut sender = NODE_SERVICE_SENDER.write().unwrap();
                *sender = None;
            }
        }
        None => warn!("NodeJs Service not connected"),
    }
}

pub async fn handle_binary_message(data: &[u8], connected_clients: &ConnectedClients) {
    let messages = ZerodhaMessage::parse(data);
    for message in messages {
        let message_json = serde_json::to_string(&message).unwrap();
        send_to_node_service_idsender(message_json.clone());

        if let Some(clients) = connected_clients
            .lock()
            .await
            .get_mut(&message.instrument_token)
        {
            clients.retain_mut(|client_tx| {
                if !client_tx.is_closed() {
                    if let Err(e) = client_tx.start_send_wrapped(message_json.clone()) {
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

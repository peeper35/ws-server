use super::dto::ZerodhaMessage;
use crate::{
    types::{ConnectedClients, NODE_SERVICE_SENDER},
    utils::ensure_nodejs_ws_connection,
};
use serde_json::json;

// async fn send_to_node_service_idsender(message: String) {
//     let mut node_service_id_sender = {
//         let sender = NODE_SERVICE_SENDER.read().unwrap();
//         sender.clone()
//     };

//     match node_service_id_sender {
//         Some(ref mut id_sender) => {
//             if let Err(e) = id_sender.start_send_wrapped(message.clone()) {
//                 warn!(
//                     "[{}] :: Failed to send message to NodeJs Service: {:?}",
//                     id_sender.uuid, e
//                 );

//                 let mut sender = NODE_SERVICE_SENDER.write().unwrap();
//                 *sender = None;
//             }
//         }
//         None => {
//             warn!("NodeJs Service not connected... Trying to restart nodejs ws connection");

//             ensure_nodejs_ws_connection().await;
//         }
//     }
// }

async fn send_to_node_service_idsender(message: String) {
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

                {
                    let mut sender = NODE_SERVICE_SENDER.write().unwrap();
                    *sender = None;
                }

                ensure_nodejs_ws_connection().await;

                node_service_id_sender = {
                    let sender = NODE_SERVICE_SENDER.read().unwrap();
                    sender.clone()
                };

                if let Some(ref mut id_sender) = node_service_id_sender {
                    if let Err(retry_error) = id_sender.start_send_wrapped(message.clone()) {
                        warn!(
                            "[{}] :: Retried but failed again to send message to NodeJs Service: {:?}",
                            id_sender.uuid, retry_error
                        );
                    } else {
                        info!(
                            "[{}] :: Successfully sent message after reconnection",
                            id_sender.uuid
                        );
                    }
                } else {
                    warn!(
                        "Failed to send message after reconnection: NodeJs Service not connected"
                    );
                }
            }
        }
        None => {
            warn!("NodeJs Service not connected... Trying to restart nodejs ws connection");

            ensure_nodejs_ws_connection().await;
        }
    }
}

pub async fn handle_binary_message(data: &[u8], connected_clients: &ConnectedClients) {
    let messages = ZerodhaMessage::parse(data);
    for message in messages {
        let message_json = serde_json::to_string(&message).unwrap();
        tokio::spawn({
            let message_json = message_json.clone();
            async move {
                send_to_node_service_idsender(message_json).await;
            }
        });
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

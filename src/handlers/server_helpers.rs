use serde_json::json;
use tokio_tungstenite::tungstenite::handshake::server::Request;
use tokio_tungstenite::tungstenite::http::Response;
use tokio_tungstenite::tungstenite::http::StatusCode;

use crate::channel::IdSender;
use crate::types::ConnectedClients;

#[instrument(skip(response))]
pub fn authenticate_websocket_connection(
    request: &Request,
    mut response: Response<()>,
) -> Result<Response<()>, Response<Option<String>>> {
    if let Some(token) = extract_token_from_request(request) {
        response
            .headers_mut()
            .insert("Sec-WebSocket-Protocol", token.parse().unwrap());
        Ok(response)
    } else {
        warn!("Unauthenticated client connect request. Closing connection");
        Err(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Some(String::from("No token found in request")))
            .unwrap())
    }
}

pub fn extract_token_from_request(request: &Request) -> Option<String> {
    request
        .headers()
        .get("Sec-WebSocket-Protocol")
        .and_then(|token| token.to_str().ok())
        .map(|token_str| token_str.to_string())
}

// use hmac::{Hmac, Mac};
// use jwt::VerifyWithKey;
// use sha2::Sha256;
// use std::collections::BTreeMap;

// use super::ws_client;

// pub async fn verify_jwt(token: &str) {
//     let key: Hmac<Sha256> = Hmac::new_from_slice(b"thisisasamplesecret").unwrap();
//     let claims: BTreeMap<String, String> = token.verify_with_key(&key).unwrap();

//     println!("JWT: {:?}", claims);
// }

pub async fn register_client_for_instruments(
    instruments: &[i32],
    connected_clients: &ConnectedClients,
    zerodha_recv_tx: IdSender,
) {
    let mut clients = connected_clients.lock().await;

    for &instrument in instruments {
        let entry = clients.entry(instrument).or_insert_with(Vec::new);

        if !entry
            .iter()
            .any(|existing| existing.uuid == zerodha_recv_tx.uuid)
        {
            entry.push(zerodha_recv_tx.clone());
        }
    }
}

pub fn create_zerodha_message(client_receive_ins: &[i32]) -> String {
    json!({ "a": "mode", "v": ["full", client_receive_ins] }).to_string()
}

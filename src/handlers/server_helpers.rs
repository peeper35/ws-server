use std::collections::HashMap;

use jwt::Error;
use serde_json::json;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::handshake::server::Request;
use tokio_tungstenite::tungstenite::http::Response;
use tokio_tungstenite::tungstenite::http::StatusCode;

use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use sha2::Sha256;

use crate::channel::IdSender;
use crate::types::ConnectedClients;
use crate::types::NODE_SERVICE_SENDER;
use crate::utils::get_env_var;

async fn parse_headers(
    stream: &mut TcpStream,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];
    let mut headers = HashMap::new();

    let bytes_read = stream.read(&mut buffer).await?;

    let mut httparse_headers = [httparse::EMPTY_HEADER; 16];
    let mut request = httparse::Request::new(&mut httparse_headers);

    match request.parse(&buffer[..bytes_read])? {
        httparse::Status::Complete(_) => {
            for header in request.headers {
                headers.insert(
                    header.name.to_string(),
                    String::from_utf8_lossy(header.value).to_string(),
                );
            }
        }
        httparse::Status::Partial => {
            warn!("Incomplete Request.")
        }
    }

    Ok(headers)
}

pub async fn store_node_service_idsender(raw_stream: &mut TcpStream, id_sender: IdSender) {
    let headers = parse_headers(raw_stream).await.unwrap();

    let token = headers.get("Sec-WebSocket-Protocol");

    match token {
        Some(token) => {
            let node_service_auth_token = get_env_var("NODE_JS_SERVICE_AUTH_TOKEN");

            if token.to_owned() == node_service_auth_token {
                let mut sender = NODE_SERVICE_SENDER.write().unwrap();
                *sender = Some(id_sender);
            }
        }
        None => (),
    }
}

#[instrument(skip(response))]
pub fn authenticate_websocket_connection(
    request: &Request,
    mut response: Response<()>,
) -> Result<Response<()>, Response<Option<String>>> {
    // if let Some(token) = extract_token_from_request(request) {
    //     response
    //         .headers_mut()
    //         .insert("Sec-WebSocket-Protocol", token.parse().unwrap());

    //     if token == get_env_var("NODE_JS_SERVICE_AUTH_TOKEN")
    //         || token == get_env_var("ORDER_MANAGEMENT_SERVICE_AUTH_TOKEN")
    //     {
    //         Ok(response)
    //     } else {
    //         if let Err(err) = verify_jwt(token.as_str()) {
    //             warn!("JWT Verify Error: {:?}", err);
    //             warn!("Unauthenticated client connect request. Closing connection");

    //             Err(Response::builder()
    //                 .status(StatusCode::UNAUTHORIZED)
    //                 .body(Some(String::from("No token found in request")))
    //                 .unwrap())
    //         } else {
    //             Ok(response)
    //         }
    //     }
    // } else {
    //     warn!("Unauthenticated client connect request. Closing connection");
    //     Err(Response::builder()
    //         .status(StatusCode::UNAUTHORIZED)
    //         .body(Some(String::from("No token found in request")))
    //         .unwrap())
    // }

    Ok(response)
}

pub fn extract_token_from_request(request: &Request) -> Option<String> {
    request
        .headers()
        .get("Sec-WebSocket-Protocol")
        .and_then(|token| token.to_str().ok())
        .map(|token_str| token_str.to_string())
}

pub fn verify_jwt(token: &str) -> Result<(), Error> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(get_env_var("JWT_SECRET").as_bytes())?;
    token.verify_with_key(&key)?;

    Ok(())
}

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

use std::sync::Arc;

use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;
use futures_util::SinkExt;
use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use crate::channel::IdReceiver;
use crate::channel::IdSender;
use crate::handlers::server_helpers::check_and_store_node_service_idsender;
use crate::handlers::server_helpers::extract_token_from_request;
use crate::types::ConnectedClients;
use crate::types::ZerodhaClientSink;
use crate::utils::get_env_var;

use tracing::instrument;

use super::server_helpers::create_zerodha_message;
use super::server_helpers::register_client_for_instruments;

use tokio_tungstenite::tungstenite::handshake::server::Request;
use tokio_tungstenite::tungstenite::http::Response;

#[instrument(skip(raw_stream, zerodha_client_sink, client_recv_rx, client_recv_tx, connected_clients), fields(conn_uuid = client_recv_rx.uuid))]
pub async fn handle_aznxt_server_conn(
    raw_stream: TcpStream,
    zerodha_client_sink: Arc<Mutex<ZerodhaClientSink>>,
    mut client_recv_rx: IdReceiver,
    client_recv_tx: IdSender,
    connected_clients: ConnectedClients,
) -> Result<(), Error> {
    let mut req_token = String::new();

    let copy_token_and_auth = |request: &Request,
                               mut response: Response<()>|
     -> Result<Response<()>, Response<Option<String>>> {
        if let Some(token) = extract_token_from_request(request) {
            response
                .headers_mut()
                .insert("Sec-WebSocket-Protocol", token.parse().unwrap());

            if token == get_env_var("NODE_JS_SERVICE_AUTH_TOKEN") {
                req_token = token.clone();
                return Ok(response);
            } else if token == get_env_var("ORDER_MANAGEMENT_SERVICE_AUTH_TOKEN") {
                return Ok(response);
            } else {
                // if let Err(err) = verify_jwt(token.as_str()) {
                //     warn!("JWT Verify Error: {:?}", err);
                //     warn!("Unauthenticated client connect request. Closing connection");

                //     return Err(Response::builder()
                //         .status(StatusCode::UNAUTHORIZED)
                //         .body(Some(String::from("No token found in request")))
                //         .unwrap());
                // } else {
                //     return Ok(response);
                // }
            }
        }
        // else {
        //     warn!("Unauthenticated client connect request. Closing connection");
        //     return Err(Response::builder()
        //         .status(StatusCode::UNAUTHORIZED)
        //         .body(Some(String::from("No token found in request")))
        //         .unwrap());
        // }

        Ok(response)
    };

    let ws_stream = accept_hdr_async(raw_stream, copy_token_and_auth).await?;

    info!("ws connection established");

    check_and_store_node_service_idsender(req_token, client_recv_tx.clone());

    let (mut client_conn_sink, client_conn_stream) = ws_stream.split();

    tokio::spawn(handle_aznxt_client_recv(
        client_conn_stream,
        zerodha_client_sink,
        client_recv_tx.clone(),
        connected_clients.clone(),
    ));

    handle_aznxt_client_send(&mut client_conn_sink, &mut client_recv_rx).await?;

    Ok(())
}

async fn handle_aznxt_client_recv(
    mut client_conn_stream: SplitStream<WebSocketStream<TcpStream>>,
    zerodha_client_sink: Arc<Mutex<ZerodhaClientSink>>,
    client_recv_tx: IdSender,
    connected_clients: ConnectedClients,
) {
    while let Some(Ok(msg)) = client_conn_stream.next().await {
        println!("Message recv: {:?}", msg);
        let instruments_text = msg.clone().into_text().unwrap();
        let client_receive_ins = serde_json::from_str::<Vec<i32>>(&instruments_text).unwrap();

        let zerodha_send_msg = create_zerodha_message(&client_receive_ins);

        zerodha_client_sink
            .lock()
            .await
            .send(zerodha_send_msg.into())
            .await
            .unwrap();

        register_client_for_instruments(
            &client_receive_ins,
            &connected_clients,
            client_recv_tx.clone(),
        )
        .await;
    }
}

#[instrument(skip(client_conn_sink, client_recv_rx), fields(conn_uuid = client_recv_rx.uuid))]
async fn handle_aznxt_client_send(
    client_conn_sink: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    client_recv_rx: &mut IdReceiver,
) -> Result<(), Error> {
    while let Some(msg) = client_recv_rx.next().await {
        if let Err(e) = client_conn_sink.send(msg.into()).await {
            info!("client disconnected");
            return Err(e);
        }
    }
    Ok(())
}

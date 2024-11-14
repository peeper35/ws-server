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
use crate::handlers::server_helpers::authenticate_websocket_connection;
use crate::handlers::server_helpers::store_node_service_idsender;
use crate::types::ConnectedClients;
use crate::types::ZerodhaClientSink;

use tracing::instrument;

use super::server_helpers::create_zerodha_message;
use super::server_helpers::register_client_for_instruments;

#[instrument(skip(raw_stream, zerodha_client_sink, client_recv_rx, client_recv_tx, connected_clients), fields(conn_uuid = client_recv_rx.uuid))]
pub async fn handle_aznxt_server_conn(
    mut raw_stream: TcpStream,
    zerodha_client_sink: Arc<Mutex<ZerodhaClientSink>>,
    mut client_recv_rx: IdReceiver,
    client_recv_tx: IdSender,
    connected_clients: ConnectedClients,
) -> Result<(), Error> {
    store_node_service_idsender(&mut raw_stream, client_recv_tx.clone()).await;

    let ws_stream = accept_hdr_async(raw_stream, authenticate_websocket_connection).await?;

    info!("ws connection established");

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

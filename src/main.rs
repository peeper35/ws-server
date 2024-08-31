mod config;
mod handlers;
mod types;
mod utils;
mod websocket_client;

use dotenv::dotenv;
use tokio::net::TcpListener;

use crate::handlers::ws_server::handle_ws_server;
use crate::websocket_client::connection::create_shared_websocket_client;

#[macro_use]
extern crate tracing;

const WS_ADDRESS: &str = "127.0.0.1:9000";

#[tokio::main]
async fn main() {
    dotenv().ok();
    config::init::init_subscriber();

    let listener = TcpListener::bind(WS_ADDRESS).await.unwrap();
    info!("WS server listening on: {}", WS_ADDRESS);

    let shared_websocket_client = create_shared_websocket_client().await;

    let (ws_conn_send_tx, ws_client_send_rx) = futures_channel::mpsc::unbounded::<String>();
    let (ws_client_recv_tx, ws_conn_recv_rx) = futures_channel::mpsc::unbounded::<String>();

    tokio::spawn(handlers::ws_client::handle_websocket_client(
        shared_websocket_client,
        ws_client_send_rx,
        ws_client_recv_tx,
    ));

    let shared_ws_conn_recv_rx = std::sync::Arc::new(tokio::sync::Mutex::new(ws_conn_recv_rx));
    let shared_ws_conn_send_tx = std::sync::Arc::new(tokio::sync::Mutex::new(ws_conn_send_tx));

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_ws_server(
            stream,
            addr,
            std::sync::Arc::clone(&shared_ws_conn_send_tx),
            std::sync::Arc::clone(&shared_ws_conn_recv_rx),
        ));
    }
}

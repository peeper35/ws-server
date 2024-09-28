mod channel;
mod config;
mod handlers;
mod types;
mod utils;
mod zerodha_conn;

use std::collections::HashMap;
use std::sync::Arc;

use channel::create_id_channels;
use dotenv::dotenv;
use handlers::aznxt_server::handle_aznxt_server_conn;
use handlers::zerodha_client::handle_zerodha_recv;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use utils::get_env_var;

use crate::zerodha_conn::connection::create_zerodha_client_split;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() {
    dotenv().ok();
    config::init::init_subscriber();

    let port = get_env_var("PORT").parse::<u32>().unwrap();

    let ws_server_addr = format!("127.0.0.1:{}", port);

    let listener = TcpListener::bind(ws_server_addr.as_str()).await.unwrap();
    info!("ws server listening on: {}", ws_server_addr);

    let (zerodha_ws_client_tx, zerodha_ws_client_rx) = create_zerodha_client_split().await;
    let connectd_clients = Arc::new(Mutex::new(HashMap::new()));

    tokio::spawn(handle_zerodha_recv(
        zerodha_ws_client_rx,
        connectd_clients.clone(),
    ));

    while let Ok((raw_stream, _)) = listener.accept().await {
        let (client_recv_tx, client_recv_rx) = create_id_channels();
        tokio::spawn(handle_aznxt_server_conn(
            raw_stream,
            zerodha_ws_client_tx.clone(),
            client_recv_rx,
            client_recv_tx,
            connectd_clients.clone(),
        ));
    }
}

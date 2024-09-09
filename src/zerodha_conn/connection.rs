use std::sync::Arc;

use futures_util::StreamExt;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;

use crate::{
    types::{ZerodhaClientSink, ZerodhaClientStream},
    utils::get_env_var,
};

pub async fn create_zerodha_client_split() -> (Arc<Mutex<ZerodhaClientSink>>, ZerodhaClientStream) {
    let zerodha_api_key = get_env_var("ZERODHA_API_KEY");
    let zerodha_access_token = get_env_var("ZERODHA_ACCESS_TOKEN");

    let ws_url = format!(
        "wss://ws.kite.trade/?api_key={}&access_token={}",
        zerodha_api_key, zerodha_access_token
    );

    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    let (stream_out, stream_in) = ws_stream.split();

    (Arc::new(Mutex::new(stream_out)), stream_in)
}

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::types::{SharedWebsocketClient, WsStreamIn, WsStreamOut};

pub struct WebsocketClient {
    stream_out: WsStreamOut,
    stream_in: WsStreamIn,
}

impl WebsocketClient {
    pub async fn connect(url: &str) -> SharedWebsocketClient {
        let (ws_stream, _) = connect_async(url)
            .await
            .expect("Failed to connect to WebSocket");

        let (stream_out, stream_in) = ws_stream.split();

        Arc::new(Mutex::new(Self {
            stream_out,
            stream_in,
        }))
    }

    // pub async fn subscribe_instrument(&mut self, exchange: &str, instrument_identifier: &str) {
    //     let subscribe_message = SubscribeRealtime::new(exchange, instrument_identifier).serialize();

    //     self.send_message(subscribe_message).await;
    // }

    pub async fn recv_next(
        &mut self,
    ) -> Option<Result<Message, tokio_tungstenite::tungstenite::Error>> {
        self.stream_in.next().await
    }

    pub async fn send_message(&mut self, message: String) {
        self.stream_out
            .send(Message::Text(message))
            .await
            .expect("Failed to send message over WebSocket");
    }
}

use std::sync::Arc;

use futures_util::stream::{SplitSink, SplitStream};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::websocket_client::client::WebsocketClient;

pub type WsStreamOut = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub type WsStreamIn = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub type SharedWebsocketClient = Arc<Mutex<WebsocketClient>>;

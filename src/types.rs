use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use lazy_static::lazy_static;

use futures_util::stream::{SplitSink, SplitStream};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::channel::IdSender;

pub type ZerodhaClientSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub type ZerodhaClientStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub type ConnectedClients = Arc<Mutex<HashMap<i32, Vec<IdSender>>>>;
pub type NodeServiceSender = RwLock<Option<IdSender>>;

lazy_static! {
    pub static ref NODE_SERVICE_SENDER: NodeServiceSender = RwLock::new(None);
}

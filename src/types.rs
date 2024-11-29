use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use futures_util::stream::{SplitSink, SplitStream};
use once_cell::sync::Lazy;
use reqwest::Client;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::channel::IdSender;

pub type ZerodhaClientSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub type ZerodhaClientStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub type ConnectedClients = Arc<Mutex<HashMap<i32, Vec<IdSender>>>>;
pub type NodeServiceSender = RwLock<Option<IdSender>>;

pub static NODE_SERVICE_SENDER: Lazy<NodeServiceSender> = Lazy::new(|| RwLock::new(None));

pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

use futures_channel::mpsc::{unbounded, SendError, UnboundedReceiver, UnboundedSender};
use futures_util::StreamExt;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct IdSender {
    pub uuid: String,
    pub sender: UnboundedSender<String>,
}

impl IdSender {
    pub fn new(uuid: String, sender: UnboundedSender<String>) -> Self {
        Self { uuid, sender }
    }
}

impl IdSender {
    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }

    pub fn start_send_wrapped(&mut self, message: String) -> Result<(), SendError> {
        self.sender.start_send(message)
    }
}

#[derive(Debug)]
pub struct IdReceiver {
    #[allow(dead_code)]
    pub uuid: String,
    pub receiver: UnboundedReceiver<String>,
}

impl IdReceiver {
    pub fn new(uuid: String, receiver: UnboundedReceiver<String>) -> Self {
        Self { uuid, receiver }
    }
}

impl IdReceiver {
    pub async fn next(&mut self) -> Option<String> {
        self.receiver.next().await
    }
}

pub fn create_id_channels() -> (IdSender, IdReceiver) {
    let uuid = Uuid::new_v4().to_string();

    let (tx, rx) = unbounded();

    (IdSender::new(uuid.clone(), tx), IdReceiver::new(uuid, rx))
}

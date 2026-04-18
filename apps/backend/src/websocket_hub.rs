use tokio::sync::broadcast;
use serde_json::Value;

#[derive(Clone)]
pub struct WsHub {
    tx: broadcast::Sender<Value>,
}

impl WsHub {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        WsHub { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Value> {
        self.tx.subscribe()
    }

    pub fn broadcast(&self, msg: Value) {
        let _ = self.tx.send(msg);
    }
}

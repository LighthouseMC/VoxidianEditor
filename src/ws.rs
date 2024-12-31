use std::sync::Arc;
use async_std::task::spawn;
use tide_websockets::WebSocketConnection;


pub const C2S_HANDSHAKE : u8 = 0;
pub const C2S_KEEPALIVE : u8 = 1;

pub const S2C_INITIAL_STATE : u8 = 0;
pub const S2C_KEEPALIVE     : u8 = 1;


pub(crate) struct WebSocketSender {
    ws : Arc<WebSocketConnection>
}
impl WebSocketSender {
    pub fn new(ws : WebSocketConnection) -> Self { Self {
        ws : Arc::new(ws)
    } }
}
impl WebSocketSender {

    pub fn send(&self, prefix : u8, mut message : Vec<u8>) {
        message.insert(0, prefix);
        let ws = self.ws.clone();
        spawn(async move {
            let _ = ws.send_bytes(message).await;
        });
    }

}


pub(crate) struct MessageBuf {
    inner : Vec<u8>,
    head  : usize
}
impl MessageBuf {

    pub fn new() -> Self { Self {
        inner : Vec::new(),
        head  : 0
    } }

    pub fn write<I : IntoIterator<Item = u8>>(&mut self, iter : I) {
        self.inner.extend(iter);
        self.head = self.inner.len();
    }

    pub fn write_str(&mut self, s : &str) {
        self.write((s.len() as u32).to_be_bytes());
        self.write(s.bytes());
    }

    pub fn extend(&mut self, other : MessageBuf) {
        self.inner.extend(other.inner.into_iter().skip(other.head));
        self.head = self.inner.len();
    }

    pub fn into_inner(self) -> Vec<u8> { self.inner }

}

use std::sync::Arc;
use async_std::task::spawn;
use tide_websockets::WebSocketConnection;


pub const C2S_HANDSHAKE : u8 = 0;
pub const C2S_KEEPALIVE : u8 = 1;

pub const S2C_INITIAL_STATE : u8 = 0;
pub const S2C_KEEPALIVE     : u8 = 1;


pub struct WebSocketSender {
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

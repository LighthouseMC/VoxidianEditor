use std::sync::Arc;
use std::sync::atomic::{ AtomicBool, Ordering };
use std::ops::{ Deref, DerefMut };
use tide_websockets::WebSocketConnection;


#[derive(Clone)]
pub(crate) struct WebSocketContainer {
    ws     : WebSocketConnection,
    closed : Arc<AtomicBool>
}
impl WebSocketContainer {

    pub fn new(ws : WebSocketConnection) -> Self { Self {
        ws,
        closed : Arc::new(AtomicBool::new(false))
    } }

    pub fn is_closed(&self) -> bool { self.closed.load(Ordering::Relaxed) }

}
impl Deref for WebSocketContainer {
    type Target = WebSocketConnection;
    fn deref(&self) -> &Self::Target { &self.ws }
}
impl DerefMut for WebSocketContainer {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.ws }
}
impl Drop for WebSocketContainer {
    fn drop(&mut self) {
        self.closed.store(true, Ordering::Relaxed);
    }
}

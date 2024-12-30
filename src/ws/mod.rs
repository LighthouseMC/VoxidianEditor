use voxidian_database::{ DBSubserverFileID, DBFilePath };
use std::sync::mpsc;
use async_std::task::{ spawn_blocking, JoinHandle };
use rouille::websocket::{ Websocket, Message };


pub struct WSClient {
    task     : JoinHandle<()>,
    close_rx : mpsc::Receiver<()>
}
impl WSClient {

    pub(crate) fn new(socket : Websocket) -> Self {
        let (close_tx, close_rx) = mpsc::channel();
        let mut task = WSClientTask {
            socket,
            close_tx
        };
        Self {
            task     : spawn_blocking(move || task.run()),
            close_rx
        }
    }

    /// This will only return `true` once. Any checks after will get `false`.
    pub fn just_closed(&self) -> bool {
        matches!(self.close_rx.try_recv(), Ok(_))
    }

}


struct WSClientTask {
    socket   : Websocket,
    close_tx : mpsc::Sender<()>
}
impl WSClientTask {


    fn run(&mut self) {
        while let Some(message) = self.socket.next() {
            let Message::Binary(prefixed_data) = message else { return };
            let prefix = prefixed_data[0];
            let data   = &prefixed_data[1..];
            self.handle_c2s_order(prefix, data);
        }
        let _ = self.close_tx.send(());
    }


    fn handle_c2s_order(&self, prefix : u8, data : &[u8]) {

        if (prefix == 0) { // C2S_ORDER_REQUEST_INITIAL_STATE
            println!("Sent initial state");
        }

    }


}

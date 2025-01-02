use voxidian_database::DBSubserverID;
use std::sync::mpmc;
use std::time::Duration;
use async_std::task::{ block_on, yield_now };
use openssl::rand::rand_priv_bytes;
use uuid::Uuid;


pub(crate) enum EditorHandleIncomingEvent {

    StartSession {
        subserver    : DBSubserverID,
        timeout      : Duration,
        player_uuid  : Uuid,
        player_name  : String,
        session_code : String
    },

    Stop

}


pub(crate) enum EditorHandleOutgoingEvent {

    Stop

}


pub struct EditorHandle {
    pub(crate) handle_incoming_tx : mpmc::Sender<EditorHandleIncomingEvent>,
    pub(crate) handle_outgoing_rx : mpmc::Receiver<EditorHandleOutgoingEvent>
}
impl EditorHandle {

    /// Starts an editor session for the given subserver.
    /// The connection will be displayed in editor as the given display name.
    /// If a connection is not established within the given timeout duration, the session is cancelled.
    pub fn start_session<const PW_LEN : usize>(&self, subserver : DBSubserverID, timeout : Duration, player_uuid : Uuid, player_name : String) -> String {
        let mut bytes = [0; PW_LEN];
        rand_priv_bytes(&mut bytes).unwrap();
        let session_code = bytes.map(|byte| rand_byte_to_char(byte)).into_iter().collect::<String>();
        let _ = self.handle_incoming_tx.send(EditorHandleIncomingEvent::StartSession {
            subserver,
            timeout,
            player_uuid,
            player_name,
            session_code : session_code.clone()
        });
        session_code
    }

}
impl Drop for EditorHandle {
    fn drop(&mut self) {
        let _ = self.handle_incoming_tx.send(EditorHandleIncomingEvent::Stop);
        loop {
            if let Ok(EditorHandleOutgoingEvent::Stop) = self.handle_outgoing_rx.try_recv() { break; }
            block_on(yield_now());
        }
    }
}


fn rand_byte_to_char(byte : u8) -> char {
    let byte = byte % 64;
    let ascii = if ((0..26).contains(&byte)) {
        65 + (byte - 0)
    } else if ((26..52).contains(&byte)) {
        97 + (byte - 26)
    } else if ((52..62).contains(&byte)) {
        48 + (byte - 52)
    } else if (byte == 62) {
        43
    } else {
        45
    };
    ascii as char
}

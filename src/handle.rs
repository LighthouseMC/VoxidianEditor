use voxidian_database::DBSubserverID;
use std::sync::mpsc;
use std::time::Duration;
use openssl::rand::rand_priv_bytes;


#[derive(Debug)]
pub(crate) enum EditorHandleIncomingEvent {

    StartSession {
        timeout      : Duration,
        subserver    : DBSubserverID,
        display_name : String,
        session_code : String
    }

}


pub struct EditorHandle {
    pub(crate) handle_incoming_tx : mpsc::Sender<EditorHandleIncomingEvent>
}
impl EditorHandle {

    /// Starts an editor session for the given subserver.
    /// The connection will be displayed in editor as the given display name.
    /// If a connection is not established within the given timeout duration, the session is cancelled.
    pub fn start_session<const PW_LEN : usize>(&self, timeout : Duration, subserver : DBSubserverID, display_name : String) -> String {
        let mut bytes = [0; PW_LEN];
        rand_priv_bytes(&mut bytes).unwrap();
        let session_code = bytes.map(|byte| rand_byte_to_char(byte)).into_iter().collect::<String>();
        let _ = self.handle_incoming_tx.send(EditorHandleIncomingEvent::StartSession {
            timeout,
            subserver,
            display_name,
            session_code : session_code.clone()
        });
        session_code
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

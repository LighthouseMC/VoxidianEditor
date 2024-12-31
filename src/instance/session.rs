use crate::ws::*;
use voxidian_logger::trace;
use voxidian_database::DBSubserverID;
use std::time::{ Instant, Duration };
use std::hint::unreachable_unchecked;
use std::sync::mpsc::{ self, TryRecvError };


pub(super) enum EditorSessionState {

    WaitingForHandshake {
        expires_at : Instant
    },

    LoggedIn {
        incoming_message_receiver : mpsc::Receiver<(u8, Vec<u8>)>,
        outgoing_message_sender   : WebSocketSender,
        last_keepalive            : (Instant, Result<u32, u32>)
    }

}


/// A 'session' is a connection from a single client to an `EditorInstance` over websocket.
pub(super) struct EditorSession {
    pub(super) subserver    : DBSubserverID,
    pub(super) display_name : String,
    pub(super) state        : EditorSessionState
}
impl EditorSession {
    pub(super) fn new(subserver : DBSubserverID, timeout : Duration, display_name : String) -> Self { Self {
        subserver,
        display_name,
        state        : EditorSessionState::WaitingForHandshake {
            expires_at : Instant::now() + timeout
        }
    } }
}
impl EditorSession {


    pub fn update_session(&mut self) -> bool {

        // State-specific behaviour.
        let retain = match (&self.state) {
            EditorSessionState::WaitingForHandshake { expires_at, .. } => {
                Instant::now() < *expires_at
            },
            EditorSessionState::LoggedIn { .. } => self.update_loggedin_session()
        };

        // Close this session if needed.
        if (! retain) {
            trace!("Closed editor session for player {} subserver {}.", self.display_name, self.subserver);
        }
        retain
    }


    fn update_loggedin_session(&mut self) -> bool {
        let EditorSessionState::LoggedIn { incoming_message_receiver, .. } = &mut self.state else { unsafe{ unreachable_unchecked() } };

        match (incoming_message_receiver.try_recv()) {
            Ok((prefix, data)) => {
                self.handle_incoming_message(prefix, data);
            },
            Err(TryRecvError::Empty) => { },
            Err(TryRecvError::Disconnected) => {
                // If the mpsc sender disconnected, it means the websocket closed. Close this session.
                return false;
            }
        }

        let EditorSessionState::LoggedIn { outgoing_message_sender, last_keepalive, .. } = &mut self.state else { unsafe{ unreachable_unchecked() } };

        // Send a keepalive message if it has been long enough.
        if let Err(next_keepalive_idx) = last_keepalive.1 && (Instant::now() >= (last_keepalive.0 + Duration::from_millis(2500))) {
            last_keepalive.0 = Instant::now();
            last_keepalive.1 = Ok(next_keepalive_idx);
            outgoing_message_sender.send(S2C_KEEPALIVE, Vec::new());
        }

        // Close this session if needed.
        Instant::now() < (last_keepalive.0 + Duration::from_millis(2500))
    }


    fn handle_incoming_message(&mut self, prefix : u8, data : Vec<u8>) {
        let EditorSessionState::LoggedIn { outgoing_message_sender, last_keepalive, .. } = &mut self.state else { unsafe{ unreachable_unchecked() } };
        match (prefix) {

            C2S_HANDSHAKE => {
                outgoing_message_sender.send(S2C_INITIAL_STATE , Vec::new());
                outgoing_message_sender.send(S2C_KEEPALIVE     , Vec::new());
                last_keepalive.1 = Ok(0);
            },

            C2S_KEEPALIVE => { if let Ok(ping_index) = data.try_into() && let Ok(last_keepalive_idx) = last_keepalive.1 {
                    let ping_index = u32::from_be_bytes(ping_index);
                    if (last_keepalive_idx == ping_index) {
                        last_keepalive.0 = Instant::now() + Duration::from_millis(2500);
                        last_keepalive.1 = Err(last_keepalive_idx.wrapping_add(1));
                    }
            } },

            _ => { }

        }
    }


}

use crate::ws::*;
use crate::instance::EditorInstanceState;
use voxidian_database::{DBSubserverFileEntityKind, DBSubserverID};
use std::time::{ Instant, Duration };
use std::sync::mpmc::{ self, TryRecvError };
use std::sync::Arc;
use async_std::sync::RwLock;
use async_std::task::{ block_on, spawn, yield_now, JoinHandle };
use uuid::Uuid;


pub(super) struct MaybePendingEditorSession {
    pub(super) subserver   : DBSubserverID,
    pub(super) player_uuid : Uuid,
    pub(super) player_name : String,
    pub(super) state       : MaybePendingEditorSessionState
}
pub(super) enum MaybePendingEditorSessionState {
    Pending {
        session_code : String,
        expires_at   : Instant
    },
    Active(EditorSessionHandle)
}


enum EditorSessionOutgoingEvent {

    Stop

}
enum EditorSessionIncomingEvent {

    Stop

}

pub(super) struct EditorSessionHandle {
    handle            : Option<JoinHandle<()>>,
    outgoing_event_tx : mpmc::Sender<EditorSessionOutgoingEvent>,
    incoming_event_rx : mpmc::Receiver<EditorSessionIncomingEvent>,
    stopped           : bool
}
impl EditorSessionHandle {
    pub fn new(
        subserver : DBSubserverID, state : Arc<RwLock<EditorInstanceState>>,
        incoming_message_rx : mpmc::Receiver<(u8, Vec<u8>)>,
        outgoing_message_tx : WebSocketSender
    ) -> Self {
        let (outgoing_event_tx, outgoing_event_rx) = mpmc::channel();
        let (incoming_event_tx, incoming_event_rx) = mpmc::channel();
        let mut session = EditorSession::new(subserver, state,
            incoming_message_rx, outgoing_message_tx,
            outgoing_event_rx, incoming_event_tx
        );
        Self {
            handle      : Some(spawn(async move {
                let _ = session.run().await;
            })),
            outgoing_event_tx,
            incoming_event_rx,
            stopped     : false
        }
    }
}
impl EditorSessionHandle {

    pub fn stop(&self) {
        let _ = self.outgoing_event_tx.send(EditorSessionOutgoingEvent::Stop);
    }

    pub fn can_drop(&self) -> bool { self.stopped }

}
impl EditorSessionHandle {

    pub fn update(&mut self) {
        if (! self.stopped) {
            while let Ok(event) = self.incoming_event_rx.try_recv() { match (event) {

                EditorSessionIncomingEvent::Stop => {
                    self.stopped = true;
                    break;
                }

            } }
        }
    }

}
impl Drop for EditorSessionHandle {
    fn drop(&mut self) {
        self.stop();
        block_on(async {
            self.handle.take().unwrap().await;
            loop { if (self.can_drop()) { break; } }
        })
    }
}


/// A 'session' is a connection from a single client to an `EditorInstance` over websocket.
pub(super) struct EditorSession {
    subserver           : DBSubserverID,
    last_keepalive      : (Instant, Result<u32, u32>),
    state               : Arc<RwLock<EditorInstanceState>>,
    incoming_message_rx : mpmc::Receiver<(u8, Vec<u8>)>,
    outgoing_message_tx : WebSocketSender,
    outgoing_event_rx   : mpmc::Receiver<EditorSessionOutgoingEvent>,
    incoming_event_tx   : mpmc::Sender<EditorSessionIncomingEvent>
}
impl EditorSession {
    fn new(
        subserver           : DBSubserverID,
        state               : Arc<RwLock<EditorInstanceState>>,
        incoming_message_rx : mpmc::Receiver<(u8, Vec<u8>)>,
        outgoing_message_tx : WebSocketSender,
        outgoing_event_rx   : mpmc::Receiver<EditorSessionOutgoingEvent>,
        incoming_event_tx   : mpmc::Sender<EditorSessionIncomingEvent>
    ) -> Self { Self {
        subserver,
        last_keepalive      : (Instant::now(), Err(0)),
        state,
        incoming_message_rx,
        outgoing_message_tx,
        outgoing_event_rx,
        incoming_event_tx
    } }
}
impl EditorSession {


    pub async fn run(&mut self) {
        loop {

            // Handle incoming messages.
            match (self.incoming_message_rx.try_recv()) {
                Ok((prefix, data)) => {
                    self.handle_incoming_message(prefix, data).await;
                },
                Err(TryRecvError::Empty) => { },
                Err(TryRecvError::Disconnected) => {
                    let _ = self.incoming_event_tx.send(EditorSessionIncomingEvent::Stop);
                    return;
                }
            }

            // Send a keepalive message if it has been long enough.
            if let Err(next_keepalive_idx) = self.last_keepalive.1 && (Instant::now() >= (self.last_keepalive.0 + Duration::from_millis(2500))) {
                self.last_keepalive.0 = Instant::now();
                self.last_keepalive.1 = Ok(next_keepalive_idx);
                self.outgoing_message_tx.send(S2C_KEEPALIVE, Vec::new());
            }

            // Close the connection if timed out.
            if let Ok(_) = self.last_keepalive.1 && (Instant::now() >= (self.last_keepalive.0 + Duration::from_millis(3750))) {
                let _ = self.incoming_event_tx.send(EditorSessionIncomingEvent::Stop);
                return;
            }

            yield_now().await;
        }
    }


    async fn handle_incoming_message(&mut self, prefix : u8, data : Vec<u8>) {
        match (prefix) {

            C2S_HANDSHAKE => {
                let mut state = self.state.write().await;
                let mut buf = MessageBuf::new();
                let properties = state.properties();
                buf.write(self.subserver.to_be_bytes());
                buf.write_str(&properties.name);
                buf.write_str(&properties.description);
                buf.write_str(&properties.owner_name);
                let file_entities = state.file_entities();
                buf.write((file_entities.len() as u32).to_be_bytes());
                for (file_entity_id, (file_path, kind)) in file_entities {
                    buf.write(file_entity_id.to_be_bytes());
                    buf.write([matches!(kind, DBSubserverFileEntityKind::Directory) as u8]);
                    buf.write_str(&file_path.to_string());
                }
                self.outgoing_message_tx.send(S2C_INITIAL_STATE, buf.into_inner());
                self.outgoing_message_tx.send(S2C_KEEPALIVE, Vec::new());
                self.last_keepalive = (Instant::now(), Ok(0));
            },

            C2S_KEEPALIVE => { if let Ok(ping_index) = data.try_into() && let Ok(last_keepalive_idx) = self.last_keepalive.1 {
                    let ping_index = u32::from_be_bytes(ping_index);
                    if (last_keepalive_idx == ping_index) {
                        self.last_keepalive.0 = Instant::now() + Duration::from_millis(2500);
                        self.last_keepalive.1 = Err(last_keepalive_idx.wrapping_add(1));
                    }
            } },

            _ => {
                voxidian_logger::error!("{} {:?}", prefix, data);
            }

        }
    }


}

use crate::ws::WebSocketContainer;
use crate::instance::EditorInstanceState;
use voxidian_editor_common::packet::{ PacketBuf, PrefixedPacketEncode, PrefixedPacketDecode };
use voxidian_editor_common::packet::s2c::*;
use voxidian_editor_common::packet::c2s::C2SPackets;
use voxidian_database::{ DBSubserverFileEntityKind, DBSubserverID };
use std::time::{ Instant, Duration };
use std::sync::mpmc;
use std::sync::Arc;
use async_std::sync::RwLock;
use async_std::stream::StreamExt;
use async_std::task::{ block_on, spawn, yield_now, JoinHandle };
use async_std::future::timeout;
use tide_websockets::Message;
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
    pub fn new(subserver : DBSubserverID, state : Arc<RwLock<EditorInstanceState>>, ws : WebSocketContainer) -> Self {
        let (outgoing_event_tx, outgoing_event_rx) = mpmc::channel();
        let (incoming_event_tx, incoming_event_rx) = mpmc::channel();
        let mut session = EditorSession::new(subserver, state, ws,
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
    subserver         : DBSubserverID,
    last_keepalive    : (Instant, Result<u64, u64>),
    state             : Arc<RwLock<EditorInstanceState>>,
    ws                : WebSocketContainer,
    outgoing_event_rx : mpmc::Receiver<EditorSessionOutgoingEvent>,
    incoming_event_tx : mpmc::Sender<EditorSessionIncomingEvent>
}
impl EditorSession {
    fn new(
        subserver         : DBSubserverID,
        state             : Arc<RwLock<EditorInstanceState>>,
        ws                : WebSocketContainer,
        outgoing_event_rx : mpmc::Receiver<EditorSessionOutgoingEvent>,
        incoming_event_tx : mpmc::Sender<EditorSessionIncomingEvent>
    ) -> Self { Self {
        subserver,
        last_keepalive    : (Instant::now(), Ok(0)),
        state,
        ws,
        outgoing_event_rx,
        incoming_event_tx
    } }
}
impl EditorSession {


    pub async fn run(&mut self) -> Result<(), ()> {
        self.send(LoginSuccessS2CPacket).await?;
        self.send(KeepaliveS2CPacket).await?;
        {
            let mut state = self.state.write().await;
            let properties = state.properties();
            self.send(InitialStateS2CPacket {
                subserver_id          : self.subserver,
                subserver_name        : properties.name.clone(),
                subserver_owner_name  : properties.owner_name.clone(),
                subserver_description : properties.description.clone(),
                file_entities         : {
                    let     file_entities = state.file_entities();
                    let mut out           = Vec::with_capacity(file_entities.len());
                    for (id, (path, kind)) in file_entities {
                        out.push(FileTreeEntry {
                            id     : *id,
                            is_dir : matches!(kind, DBSubserverFileEntityKind::Directory),
                            path   : path.to_string()
                        })
                    }
                    out
                },
            }).await?;
        }
        loop {

            // Handle events.
            while let Ok(events) = self.outgoing_event_rx.try_recv() { match (events) {
                EditorSessionOutgoingEvent::Stop => {
                    self.send(DisconnectS2CPacket { reason : "Shutting down".to_string() }).await?;
                    self.stop()?;
                }
            } }

            // Handle incoming messages.
            match (timeout(Duration::ZERO, self.ws.next()).await) {
                Err(_) => { },
                Ok(None) => { self.stop()?; },
                Ok(Some(Err(err))) => {
                    self.send(DisconnectS2CPacket { reason : format!("An error occured: {}", err) }).await?;
                    self.stop()?;
                },
                Ok(Some(Ok(Message::Binary(data)))) => {
                    match (C2SPackets::decode_prefixed(&mut PacketBuf::from(data))) {
                        Err(err) => {
                            self.send(DisconnectS2CPacket { reason : format!("An error occured: {:?}", err) }).await?;
                            self.stop()?;
                        },
                        Ok(packet) => { self.handle_incoming_message(packet).await?; }
                    }
                },
                Ok(Some(Ok(_))) => {
                    self.send(DisconnectS2CPacket { reason : "Bad data format".to_string() }).await?;
                    self.stop()?;
                }
            }

            // Send a keepalive message if it has been long enough.
            if let Err(next_keepalive_idx) = self.last_keepalive.1 && (Instant::now() >= (self.last_keepalive.0 + Duration::from_millis(1500))) {
                self.last_keepalive.0 = Instant::now();
                self.last_keepalive.1 = Ok(next_keepalive_idx);
                self.send(KeepaliveS2CPacket).await?;
            }

            // Close the connection if timed out.
            if let Ok(_) = self.last_keepalive.1 && (Instant::now() >= (self.last_keepalive.0 + Duration::from_millis(2500))) {
                self.send(DisconnectS2CPacket { reason : "Timed out".to_string() }).await?;
                self.stop()?;
            }

            yield_now().await;
        }
    }


    async fn send<P : PrefixedPacketEncode>(&self, packet : P) -> Result<(), ()> {
        if let Err(err) = self.ws.send_bytes(PacketBuf::of_encode_prefixed(packet).into_inner()).await {
            let _ = self.ws.send_bytes(PacketBuf::of_encode_prefixed(DisconnectS2CPacket { reason : format!("An error occured: {}", err) }).into_inner()).await;
            self.stop()
        } else { Ok(()) }
    }


    fn stop(&self) -> Result<(), ()> {
        let _ = self.incoming_event_tx.send(EditorSessionIncomingEvent::Stop);
        Err(())
    }


    async fn handle_incoming_message(&mut self, packet : C2SPackets) -> Result<(), ()> {
        match (packet) {

            C2SPackets::Handshake(_) => {
                self.send(DisconnectS2CPacket { reason : "An error occured: Out of order handshake".to_string() }).await?;
                self.stop()?;
            },

            C2SPackets::Keepalive(keepalive) => { match (self.last_keepalive.1) {
                Ok(expected_keepalive_index) => {
                    if (keepalive.index == expected_keepalive_index) {
                        self.last_keepalive.0 = Instant::now() + Duration::from_millis(1500);
                        self.last_keepalive.1 = Err(expected_keepalive_index.wrapping_add(1));
                    } else {
                        self.send(DisconnectS2CPacket { reason : "An error occured: Out of order ping".to_string() }).await?;
                        self.stop()?;
                    }
                },
                Err(_) => {
                    self.send(DisconnectS2CPacket { reason : "An error occured: Out of order ping".to_string() }).await?;
                    self.stop()?;
                },
            } }

        }
        Ok(())
    }


}

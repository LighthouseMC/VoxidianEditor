use crate::ws::WebSocketContainer;
use crate::instance::EditorInstanceState;
use voxidian_editor_common::packet::{ PacketBuf, PrefixedPacketEncode, PrefixedPacketDecode };
use voxidian_editor_common::packet::s2c::*;
use voxidian_editor_common::packet::c2s::C2SPackets;
use voxidian_editor_common::dmp::{ DiffMatchPatch, Patches, PatchInput, Efficient };
use voxidian_database::{ DBSubserverFileEntityKind, DBSubserverFileID, DBSubserverID };
use std::time::{ Instant, Duration };
use std::sync::mpmc;
use std::sync::Arc;
use std::collections::HashMap;
use std::str;
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


#[derive(Clone, Copy)]
pub enum EditorSessionStopReason {
    ServerShutDown,
    LoggedInElsewhere,
    SessionClosed
}
impl EditorSessionStopReason {
    fn as_str(self) -> &'static str { match (self) {
        Self::ServerShutDown    => "Shutting down",
        Self::LoggedInElsewhere => "Logged in from another location",
        Self::SessionClosed     => "Session closed"
    } }
}


enum EditorSessionOutgoingEvent {

    // Sent to a session to stop it.
    Stop(EditorSessionStopReason),

    /// A file patch from the client was acknowledged. Apply it to the Server Shadow.
    PatchShadow {
        id      : DBSubserverFileID,
        patches : Patches<Efficient>
    },

    /// A new Server Text snapshot is available. Diff it against the Server Shadow and send the patches to the client.
    FileContentsPatchToClient {
        id          : DBSubserverFileID,
        server_text : String
    }

}
enum EditorSessionIncomingEvent {

    // Sent back to the instance to acknowledged a stop command.
    Stop,

    /// Some patches were received from the client. Apply it to the Server Text.
    FilePatchFromClient {
        id      : DBSubserverFileID,
        patches : Patches<Efficient>
    }

}

pub(super) struct EditorSessionHandle {
    handle            : Option<JoinHandle<()>>,
    outgoing_event_tx : mpmc::Sender<EditorSessionOutgoingEvent>,
    incoming_event_rx : mpmc::Receiver<EditorSessionIncomingEvent>,
    pending_patches   : Vec<(DBSubserverFileID, Patches<Efficient>)>,
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
            handle            : Some(spawn(async move {
                let _ = session.run().await;
            })),
            outgoing_event_tx,
            incoming_event_rx,
            pending_patches   : Vec::new(),
            stopped           : false
        }
    }
}
impl EditorSessionHandle {

    pub fn pending_patches(&mut self) -> &mut Vec<(DBSubserverFileID, Patches<Efficient>)> {
        &mut self.pending_patches
    }

    pub fn patch_file_to_client(&self, id : DBSubserverFileID, server_text : String) {
        let _ = self.outgoing_event_tx.send(EditorSessionOutgoingEvent::FileContentsPatchToClient {
            id,
            server_text
        });
    }

    pub fn stop(&self, reason : EditorSessionStopReason) {
        let _ = self.outgoing_event_tx.send(EditorSessionOutgoingEvent::Stop(reason));
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
                },

                EditorSessionIncomingEvent::FilePatchFromClient { id, patches } => {
                    self.pending_patches.push((id, patches.clone())); // TODO: Use these
                    let _ = self.outgoing_event_tx.send(EditorSessionOutgoingEvent::PatchShadow { id, patches });
                }

            } }
        }
    }

}
impl Drop for EditorSessionHandle {
    fn drop(&mut self) {
        self.stop(EditorSessionStopReason::SessionClosed);
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
    file_shadows      : HashMap<DBSubserverFileID, String>,
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
        file_shadows      : HashMap::new(),
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

            // Handle outgoing events.
            while let Ok(events) = self.outgoing_event_rx.try_recv() { match (events) {

                EditorSessionOutgoingEvent::Stop(reason) => {
                    self.send(DisconnectS2CPacket { reason : reason.as_str().to_string() }).await?;
                    self.stop()?;
                },

                EditorSessionOutgoingEvent::PatchShadow { id, patches } => {
                    if let Some(old_server_shadow) = self.file_shadows.get_mut(&id) {
                        let dmp = DiffMatchPatch::new();
                        if let Ok((new_server_shadow, _)) = dmp.patch_apply(&patches, &old_server_shadow) {
                            *old_server_shadow = new_server_shadow;
                        }
                    }
                },

                EditorSessionOutgoingEvent::FileContentsPatchToClient { id, server_text } => {
                    if let Some(server_shadow) = self.file_shadows.get_mut(&id) {
                        let dmp = DiffMatchPatch::new();
                        // Server Text is diffed against the Server Shadow.
                        let diffs = dmp.diff_main::<Efficient>(server_shadow, &server_text).unwrap();
                        // This returns a list of edits which have been performed on Server Text.
                        let patches = dmp.patch_make(PatchInput::new_diffs(&diffs)).unwrap();
                        // Server Text is copied over to Server Shadow.
                        *server_shadow = server_text;
                        // The edits are sent to the Client.
                        self.send(PatchFileS2CPacket {
                            id,
                            patches
                        }).await?;
                    }
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
            } },


            C2SPackets::OpenFile(open_file) => {
                let id = open_file.id;
                if let Some((_, DBSubserverFileEntityKind::File(data))) = self.state.write().await.file_entities().get(&id) {
                    if let Ok(text) = str::from_utf8(data) {
                        let _ = self.file_shadows.insert(id, text.to_string());
                        self.send(OverwriteFileS2CPacket {
                            id,
                            contents : FileContents::Text(text.to_string())
                        }).await?;
                    } else {
                        self.send(OverwriteFileS2CPacket {
                            id,
                            contents : FileContents::NonText
                        }).await?;
                    }
                }
            },


            C2SPackets::CloseFile(close_file) => {
                self.file_shadows.remove(&close_file.id);
            },


            C2SPackets::PatchFile(patch_file) => {
                let _ = self.incoming_event_tx.send(EditorSessionIncomingEvent::FilePatchFromClient {
                    id      : patch_file.id,
                    patches : patch_file.patches
                });
            }


        }
        Ok(())
    }


}

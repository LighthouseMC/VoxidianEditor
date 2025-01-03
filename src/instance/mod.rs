mod session;
use session::*;
mod state;
use state::*;


use crate::ws::WebSocketContainer;
use crate::handle::{ EditorHandleIncomingEvent, EditorHandleOutgoingEvent };
use voxidian_editor_common::packet::PacketBuf;
use voxidian_editor_common::packet::s2c::DisconnectS2CPacket;
use voxidian_editor_common::dmp::DiffMatchPatch;
use voxidian_logger::{ debug, trace };
use voxidian_database::{ DBSubserverFileEntityKind, DBSubserverID, VoxidianDB };
use std::sync::{ mpmc, Arc };
use std::collections::HashMap;
use std::time::{ Instant, Duration };
use async_std::sync::RwLock;
use async_std::task::{ block_on, spawn, yield_now };
use uuid::Uuid;


pub(crate) struct EditorInstanceManager {
    handle_incoming_rx : mpmc::Receiver<EditorHandleIncomingEvent>,
    handle_outgoing_tx : mpmc::Sender<EditorHandleOutgoingEvent>,
    add_ws_rx          : mpmc::Receiver<(WebSocketContainer, String)>,
    instances          : HashMap<DBSubserverID, EditorInstance>
}
impl EditorInstanceManager {
    pub(crate) fn new(
        handle_incoming_rx : mpmc::Receiver<EditorHandleIncomingEvent>,
        handle_outgoing_tx : mpmc::Sender<EditorHandleOutgoingEvent>,
        add_ws_rx          : mpmc::Receiver<(WebSocketContainer, String)>
    ) -> Self { Self {
        handle_incoming_rx,
        handle_outgoing_tx,
        add_ws_rx,
        instances          : HashMap::new()
    } } 
}
impl EditorInstanceManager {


    pub(crate) async fn run(&mut self, db : Arc<VoxidianDB>) {
        loop {

            // Accept commands from the main server.
            while let Ok(handle_incoming) = self.handle_incoming_rx.try_recv() { match (handle_incoming) {
                EditorHandleIncomingEvent::StartSession { timeout, subserver, player_uuid, player_name, session_code } => {
                    self.start_session(&db, subserver, timeout, player_uuid, player_name, session_code);
                },
                EditorHandleIncomingEvent::Stop => {
                    for (_, instance) in &mut self.instances {
                        instance.stop(EditorSessionStopReason::ServerShutDown).await;
                    }
                    let _ = self.handle_outgoing_tx.send(EditorHandleOutgoingEvent::Stop);
                    return;
                }
            } }

            // Remove any instances which have no sessions/
            self.instances.retain(|_, instance| instance.update_sessions());

            // Accept new session logins.
            'accept_logins : while let Ok((ws, given_session_code)) = self.add_ws_rx.try_recv() {
                'check_instances : for instance in self.instances.values_mut() {
                    for session in instance.sessions.values_mut() {
                        if let MaybePendingEditorSessionState::Pending { session_code, .. } = &session.state {
                            if (&given_session_code == session_code || true) { // TODO: Disable this `true`
                                trace!("{} logged in to editor session for subserver {}.", session.player_name, instance.subserver);
                                session.state = MaybePendingEditorSessionState::Active(EditorSessionHandle::new(
                                    session.subserver, instance.state.clone(),
                                    ws
                                ));
                                continue 'accept_logins;
                            }
                            break 'check_instances;
                        }
                    }
                }
                spawn(async move { let _ = ws.send_bytes(PacketBuf::of_encode_prefixed(DisconnectS2CPacket { reason : "Invalid session code. Has it expired?".to_string() }).into_inner()).await; });
            }

            yield_now().await;
        }
    }


    fn get_or_create_instance(&mut self, db : &Arc<VoxidianDB>, subserver : DBSubserverID) -> &mut EditorInstance {
        self.instances.entry(subserver).or_insert_with(|| {
            debug!("Opened editor instance for subserver {}.", subserver);
            EditorInstance::new(db.clone(), subserver)
        })
    }


    fn start_session(&mut self, db : &Arc<VoxidianDB>, subserver : DBSubserverID, timeout : Duration, player_uuid : Uuid, player_name : String, session_code : String) {
        let instance = self.get_or_create_instance(db, subserver);
        instance.start_session(subserver, timeout, player_uuid, player_name, session_code);
    }


}


/// An 'instance' holds the current state of a subserver's codebase.
///  Multiple sessions can connect to it.
struct EditorInstance {
    subserver    : DBSubserverID,
    sessions     : HashMap<usize, MaybePendingEditorSession>,
    next_session : usize,
    state        : Arc<RwLock<EditorInstanceState>>
}
impl EditorInstance {
    pub fn new(db : Arc<VoxidianDB>, subserver : DBSubserverID) -> Self { Self {
        subserver,
        sessions     : HashMap::new(),
        next_session : 0,
        state        : Arc::new(RwLock::new(EditorInstanceState::open(db, subserver)))
    } } 
}
impl EditorInstance {


    fn start_session(&mut self, subserver : DBSubserverID, timeout : Duration, player_uuid : Uuid, player_name : String, session_code : String) {
        // Shut down old sessions owned by this player.
        self.sessions.retain(|_, session| {
            if (session.player_uuid == player_uuid) {match (&session.state) {
                MaybePendingEditorSessionState::Pending { .. } => {
                    trace!("Closed editor session for player {} subserver {}.", session.player_name, self.subserver);
                    false
                },
                MaybePendingEditorSessionState::Active(handle) => { handle.stop(EditorSessionStopReason::LoggedInElsewhere); true },
            } } else { true }
        });
        // Start the new session.
        trace!("{} opened an editor session for subserver {}.", player_name, subserver);
        self.sessions.insert(self.next_session, MaybePendingEditorSession {
            subserver,
            player_uuid,
            player_name,
            state : MaybePendingEditorSessionState::Pending {
                session_code,
                expires_at   : Instant::now() + timeout
            }
        });
        self.next_session += 1;
    }


    fn update_sessions(&mut self) -> bool {
        // Update active sessions.
        for session in self.sessions.values_mut() {
            match (&mut session.state) {
                MaybePendingEditorSessionState::Pending { .. } =>  { },
                MaybePendingEditorSessionState::Active(handle) => {
                    handle.update();
                }
            }
        }

        // Pull together all queued patches and apply them to the Server Text.
        // TODO: Only run this ever half a second.
        let mut all_patches = HashMap::new();
        for session in self.sessions.values_mut() {
            match (&mut session.state) {
                MaybePendingEditorSessionState::Pending { .. } => { },
                MaybePendingEditorSessionState::Active(handle) => {
                    let pending_patches = handle.pending_patches();
                    for (file_id, patches) in pending_patches.drain(0..pending_patches.len()) {
                        all_patches.entry(file_id).or_insert_with(|| Vec::new()).extend(patches);
                    }
                }
            }
        }
        for (file_id, patches) in all_patches {
            if let Some((_, DBSubserverFileEntityKind::File(server_data))) = self.state.write_blocking().file_entities().get_mut(&file_id) {
                if let Ok(mut server_text) = String::from_utf8(server_data.clone()) {
                    let dmp = DiffMatchPatch::new();
                    server_text = dmp.patch_apply(&patches, &server_text).unwrap().0;
                    for (_, session) in &self.sessions {
                        match (&session.state) {
                            MaybePendingEditorSessionState::Pending { .. } => { },
                            MaybePendingEditorSessionState::Active(handle) => {
                                handle.patch_file_to_client(file_id, server_text.clone());
                            },
                        }
                    }
                    *server_data = server_text.into_bytes();
                }
            }
        }

        // Remove any sessions.
        self.sessions.retain(|_, session| {
            let retain = match (&session.state) {
                MaybePendingEditorSessionState::Pending { expires_at, .. } => { Instant::now() < *expires_at },
                MaybePendingEditorSessionState::Active(handle) => { ! handle.can_drop() },
            };
            if (! retain) {
                trace!("Closed editor session for player {} subserver {}.", session.player_name, self.subserver);
            }
            retain
        });

        // Close this instance if there are no sessions.
        let retain = ! self.sessions.is_empty();
        if (! retain) {
            debug!("Closed editor instance for subserver {}.", self.subserver);
        }
        retain
    }


    pub async fn stop(&mut self, reason : EditorSessionStopReason) {
        for (_, session) in &mut self.sessions {
            let mut can_exit = true;
            match (&mut session.state) {
                MaybePendingEditorSessionState::Pending { .. } =>  { },
                MaybePendingEditorSessionState::Active(handle) => {
                    handle.stop(reason);
                    if (! handle.can_drop()) {
                        can_exit = false;
                    }
                }
            }
            if (can_exit) { break; }
            yield_now().await;
        }
    }


}
impl Drop for EditorInstance {
    fn drop(&mut self) {
        block_on(self.stop(EditorSessionStopReason::SessionClosed));
    }
}

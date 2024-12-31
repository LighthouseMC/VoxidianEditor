mod session;
use session::*;
mod state;
use state::*;


use crate::ws::WebSocketSender;
use crate::handle::EditorHandleIncomingEvent;
use voxidian_logger::{ debug, trace };
use voxidian_database::{ VoxidianDB, DBSubserverID };
use std::sync::{ mpmc, Arc };
use std::collections::HashMap;
use std::time::{ Instant, Duration };
use async_std::sync::RwLock;
use async_std::task::yield_now;
use uuid::Uuid;


pub(crate) struct EditorInstanceManager {
    handle_incoming_rx : mpmc::Receiver<EditorHandleIncomingEvent>,
    add_ws_rx          : mpmc::Receiver<(mpmc::Receiver<(u8, Vec<u8>)>, WebSocketSender, String)>,
    instances          : HashMap<DBSubserverID, EditorInstance>
}
impl EditorInstanceManager {
    pub(crate) fn new(
        handle_incoming_rx : mpmc::Receiver<EditorHandleIncomingEvent>,
        add_ws_rx          : mpmc::Receiver<(mpmc::Receiver<(u8, Vec<u8>)>, WebSocketSender, String)>
    ) -> Self { Self {
        handle_incoming_rx,
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
            } }

            // Remove any instances which have no sessions/
            self.instances.retain(|_, instance| instance.update_sessions());

            // Accept new session logins.
            while let Ok((incoming_message_rx, outgoing_message_tx, given_session_code)) = self.add_ws_rx.try_recv() {
                'check_instances : for instance in self.instances.values_mut() {
                    for session in instance.sessions.values_mut() {
                        if let MaybePendingEditorSessionState::Pending { session_code, .. } = &session.state {
                            if (&given_session_code == session_code) {
                                trace!("{} logged in to editor session for subserver {}.", session.player_name, instance.subserver);
                                session.state = MaybePendingEditorSessionState::Active(EditorSessionHandle::new(
                                    session.subserver, instance.state.clone(),
                                    incoming_message_rx,
                                    outgoing_message_tx
                                ));
                            }
                            break 'check_instances;
                        }
                    }
                }
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
            if (session.player_uuid == player_uuid) { match (&session.state) {
                MaybePendingEditorSessionState::Pending { .. } => {
                    trace!("Closed editor session for player {} subserver {}.", session.player_name, self.subserver);
                    false
                },
                MaybePendingEditorSessionState::Active(handle) => { handle.stop(); true },
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


}

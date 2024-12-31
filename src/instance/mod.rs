mod session;
use session::*;
mod state;
use state::*;


use crate::ws::WebSocketSender;
use crate::handle::EditorHandleIncomingEvent;
use voxidian_logger::{ debug, trace };
use voxidian_database::{ VoxidianDB, DBSubserverID };
use std::sync::{ mpsc, Arc };
use std::collections::HashMap;
use std::time::Instant;
use async_std::task::{ block_on, yield_now };


pub(crate) struct EditorInstanceManager {
    handle_incoming_rx : mpsc::Receiver<EditorHandleIncomingEvent>,
    add_ws_rx          : mpsc::Receiver<(mpsc::Receiver<(u8, Vec<u8>)>, WebSocketSender, String)>,
    instances          : HashMap<DBSubserverID, EditorInstance>
}
impl EditorInstanceManager {
    pub(crate) fn new(
        handle_incoming_rx : mpsc::Receiver<EditorHandleIncomingEvent>,
        add_ws_rx          : mpsc::Receiver<(mpsc::Receiver<(u8, Vec<u8>)>, WebSocketSender, String)>
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
                EditorHandleIncomingEvent::StartSession { timeout, subserver, display_name, session_code } => {
                    self.start_session(&db, subserver, display_name.clone(), session_code, EditorSession::new(subserver, timeout, display_name));
                },
            } }

            // Remove any instances which have no sessions/
            self.instances.retain(|_, instance| instance.update_sessions());

            // Accept new session logins.
            while let Ok((incoming_message_receiver, outgoing_message_sender, session_code)) = self.add_ws_rx.try_recv() {
                'check_instances : for instance in self.instances.values_mut() { if (instance.state.is_ready()) {
                    if let Some(session) = instance.sessions.get_mut(&session_code) {
                        if let EditorSessionState::WaitingForHandshake { .. } = session.state {
                            trace!("{} logged in to editor session for subserver {}.", session.display_name, instance.subserver);
                            session.state = EditorSessionState::LoggedIn {
                                incoming_message_receiver,
                                outgoing_message_sender,
                                last_keepalive            : (Instant::now(), Err(0))
                            };
                        }
                        break 'check_instances;
                    }
                } }
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


    fn start_session(&mut self, db : &Arc<VoxidianDB>, subserver : DBSubserverID, display_name : String, session_code : String, session : EditorSession) {
        let instance = self.get_or_create_instance(db, subserver);
        instance.start_session(session_code, session);
        trace!("{} opened an editor session for subserver {}.", display_name, subserver);
    }


}


/// An 'instance' holds the current state of a subserver's codebase.
///  Multiple sessions can connect to it.
struct EditorInstance {
    subserver : DBSubserverID,
    sessions  : HashMap<String, EditorSession>,
    state     : EditorInstanceState
}
impl EditorInstance {
    pub fn new(db : Arc<VoxidianDB>, subserver : DBSubserverID) -> Self { Self {
        subserver,
        sessions  : HashMap::new(),
        state     : EditorInstanceState::open(db, subserver)
    } } 
}
impl EditorInstance {


    fn start_session(&mut self, session_code : String, session : EditorSession) {
        // If there are too many sessions, start closing any sessions that have not yet logged in.
        if (self.sessions.len() > 25) {
            self.sessions.retain(|_, session| {
                let retain = matches!(session.state, EditorSessionState::LoggedIn { .. });
                if (! retain) {
                    trace!("Closed editor session for player {} subserver {}.", session.display_name, self.subserver);
                }
                retain
            });
        }
        // Start the new session.
        self.sessions.insert(session_code, session);
    }


    fn update_sessions(&mut self) -> bool {
        // Remove any sessions.
        self.sessions.retain(|_, session| session.update_session(&mut self.state));

        // Close this instance if there are no sessions.
        let retain = ! self.sessions.is_empty();
        if (! retain) {
            debug!("Closed editor instance for subserver {}.", self.subserver);
        }
        retain
    }


}
impl Drop for EditorInstance {
    fn drop(&mut self) {
        block_on(self.state.save());
    }
}

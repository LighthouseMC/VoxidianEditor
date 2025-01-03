use voxidian_database::{ DBFilePath, DBSubserverFileEntityKind, DBSubserverFileID, DBSubserverID, VoxidianDB };
use std::time::{ Instant, Duration };
use std::sync::{ mpmc, Arc };
use std::collections::HashMap;
use async_std::task::spawn;


pub(crate) struct EditorInstanceState {
    db        : Arc<VoxidianDB>,
    subserver : DBSubserverID,

    initial_rx    : mpmc::Receiver<(
        SubserverProperties,
        HashMap<DBSubserverFileID, (DBFilePath, DBSubserverFileEntityKind)>
    )>,
    properties    : Option<SubserverProperties>,
    file_entities : Option<HashMap<DBSubserverFileID, (DBFilePath, DBSubserverFileEntityKind)>>,

    edited     : bool,
    last_saved : Instant
}
impl EditorInstanceState {

    pub fn open(db : Arc<VoxidianDB>, subserver : DBSubserverID) -> Self {
        let (initial_tx, initial_rx) = mpmc::channel();
        spawn(Self::load_files(db.clone(), subserver, initial_tx));
        Self {
            db,
            subserver,

            initial_rx,
            properties    : None,
            file_entities : None,

            edited     : false,
            last_saved : Instant::now()
        }
    }

    async fn load_files(db : Arc<VoxidianDB>, subserver : DBSubserverID, initial_tx : mpmc::Sender<(
        SubserverProperties,
        HashMap<DBSubserverFileID, (DBFilePath, DBSubserverFileEntityKind)>
    )>) {
        let db_subserver = db.get_subserver(subserver).await.unwrap().unwrap();
        let db_player    = db.get_player(db_subserver.owning_player).await.unwrap().unwrap();
        let properties = SubserverProperties {
            name      : db_subserver.title,
            description   : db_subserver.subtitle,
            owner_name : db_player.username
        };
        let mut file_entities = HashMap::new();
        for file_entity in db.get_subserver_files_of(subserver).await.unwrap() {
            file_entities.insert(file_entity.id, (file_entity.path, file_entity.kind));
        }
        let _ = initial_tx.send((properties, file_entities));
    }

    pub fn is_ready(&mut self) -> bool {
        matches!(self.file_entities_or_none(), Some(_))
    }

    pub fn properties_or_none(&mut self) -> Option<&mut SubserverProperties> {
        if (self.file_entities.is_none()) {
            if let Ok((properties, file_entities)) = self.initial_rx.try_recv() {
                self.properties    = Some(properties);
                self.file_entities = Some(file_entities);
                unsafe{ Some(self.properties.as_mut().unwrap_unchecked()) }
            } else { None }
        } else { unsafe{ Some(self.properties.as_mut().unwrap_unchecked()) } }
    }

    pub fn properties(&mut self) -> &SubserverProperties {
        self.properties_or_none().unwrap()
    }

    pub fn file_entities_or_none(&mut self) -> Option<&mut HashMap<DBSubserverFileID, (DBFilePath, DBSubserverFileEntityKind)>> {
        if (self.file_entities.is_none()) {
            if let Ok((properties, file_entities)) = self.initial_rx.try_recv() {
                self.properties    = Some(properties);
                self.file_entities = Some(file_entities);
                unsafe{ Some(self.file_entities.as_mut().unwrap_unchecked()) }
            } else { None }
        } else { unsafe{ Some(self.file_entities.as_mut().unwrap_unchecked()) } }
    }

    pub fn file_entities(&mut self) -> &mut HashMap<DBSubserverFileID, (DBFilePath, DBSubserverFileEntityKind)> {
        self.file_entities_or_none().unwrap()
    }

    pub async fn save(&mut self) {
        if (! self.edited) {
            self.last_saved = Instant::now();
            return;
        }
        self.edited = false;
        todo!();
    }

    /// Will not block at all.
    pub fn autosave(&mut self) {
        if (Instant::now() >= (self.last_saved + Duration::from_secs(5))) {
            if (! self.edited) {
                self.last_saved = Instant::now();
                return;
            }
            self.edited = false;
            todo!();
        }
    }

}


pub struct SubserverProperties {
    pub name        : String,
    pub description : String,
    pub owner_name  : String
}

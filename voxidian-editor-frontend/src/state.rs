use voxidian_editor_common::packet::s2c::{ FileTreeEntry, FileContents };
use voxidian_editor_common::packet::c2s::*;
use std::cell::LazyCell;
use std::sync::{ RwLock, RwLockReadGuard, RwLockWriteGuard };
use std::collections::HashMap;


pub static FILES : FilesContainer = FilesContainer::new();
pub struct FilesContainer {
    files : LazyCell<RwLock<HashMap<u32, FilesEntry>>>
}
impl FilesContainer { const fn new() -> Self { Self {
    files : LazyCell::new(|| RwLock::new(HashMap::new()))
} } }
impl FilesContainer {
    pub fn read(&self) -> RwLockReadGuard<HashMap<u32, FilesEntry>> {
        self.files.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<HashMap<u32, FilesEntry>> {
        self.files.write().unwrap()
    }
}
unsafe impl Sync for FilesContainer { }


#[derive(Debug)]
pub struct FilesEntry {
    pub id   : u32,
    pub path : String,
    pub kind : FilesEntryKind
}
#[derive(Debug)]
pub enum FilesEntryKind {
    Directory,
    File {
        is_open : Option<Option<FileContents>>
        //        |      |      ^- File data
        //        |      ^- None if opened but no data from server yet
        //        ^- None if file not opened
    }
}


pub fn add_file(entry : &FileTreeEntry) {
    FILES.write().insert(entry.id, FilesEntry {
        id   : entry.id,
        path : entry.path.clone(),
        kind : if (entry.is_dir) { FilesEntryKind::Directory }
        else { FilesEntryKind::File {
            is_open : None
        } }
    });
    crate::filetree::add(&entry);
}

pub fn open_file(id : u32) {
    let mut files = FILES.write();
    let Some(FilesEntry { path, kind : FilesEntryKind::File { is_open }, .. }) = files.get_mut(&id) else { return; };
    match (is_open) {
        Some(Some(FileContents::NonText)) => { crate::code::open_nontext(); },
        Some(Some(FileContents::Text(_))) => { crate::code::open_monaco(id); },
        Some(None) => { crate::code::open_load(); },
        None => {
            *is_open = Some(None);
        crate::ws::WS.send(OpenFileC2SPacket { id });
        crate::code::open_load();
        }
    }
    crate::filetree::open(&path);
    crate::filetabs::open(id, &path);
}

pub fn close_file(id : u32) {
    let mut files = FILES.write();
    let Some(FilesEntry { path, kind : FilesEntryKind::File { is_open }, .. }) = files.get_mut(&id) else { return; };
    if let Some(_) = is_open {
        *is_open = None;
        let path = path.clone();
        drop(files);
        crate::filetabs::close(&path);
        crate::ws::WS.send(CloseFileC2SPacket { id });
    }
}

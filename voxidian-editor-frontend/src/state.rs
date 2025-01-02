use voxidian_editor_common::packet::s2c::FileTreeEntry;
use std::cell::LazyCell;
use std::sync::{ RwLock, RwLockReadGuard, RwLockWriteGuard };
use std::collections::HashMap;


static FILES : FilesContainer = FilesContainer::new();
pub struct FilesContainer {
    files : LazyCell<RwLock<HashMap<u32, FilesEntry>>>
}
impl FilesContainer { const fn new() -> Self { Self {
    files : LazyCell::new(|| RwLock::new(HashMap::new()))
} } }
impl FilesContainer {
    fn read(&self) -> RwLockReadGuard<HashMap<u32, FilesEntry>> { self.files.read().unwrap() }
    fn write(&self) -> RwLockWriteGuard<HashMap<u32, FilesEntry>> { self.files.write().unwrap() }
}
unsafe impl Sync for FilesContainer { }


struct FilesEntry {
    pub id   : u32,
    pub path : String,
    pub kind : FilesEntryKind
}
enum FilesEntryKind {
    Directory,
    File {
        is_open : bool
    }
}


pub fn add_file(entry : &FileTreeEntry) {
    FILES.write().insert(entry.id, FilesEntry {
        id   : entry.id,
        path : entry.path.clone(),
        kind : if (entry.is_dir) { FilesEntryKind::Directory }
        else { FilesEntryKind::File {
            is_open : false
        } }
    });
    crate::filetree::add(&entry);
}

pub fn open_file(id : u32) {
    let mut files = FILES.write();
    let Some(FilesEntry { path, kind : FilesEntryKind::File { is_open }, .. }) = files.get_mut(&id) else { return; };
    if (! *is_open) {
        *is_open = true;
    }
    crate::filetree::open(&path);
    crate::filetabs::open(id, &path);
}

pub fn close_file(id : u32) {
    let mut files = FILES.write();
    let Some(FilesEntry { path, kind : FilesEntryKind::File { is_open }, .. }) = files.get_mut(&id) else { return; };
    if (*is_open) {
        *is_open = false;
        crate::filetabs::close(&path);
    }
}

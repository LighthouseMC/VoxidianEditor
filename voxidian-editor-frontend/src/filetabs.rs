use crate::state::{ FilesEntry, FilesEntryKind };
use voxidian_editor_common::packet::s2c::FileContents;
use wasm_bindgen::prelude::*;


pub fn open(id : u32, path : &str) {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // File tab
    let filetabs = document.get_element_by_id("editor_filetabs").unwrap();
    let children = filetabs.children();
    let mut before = None;
    let mut found  = None;
    for i in 0..children.length() {
        let tab = children.get_with_index(i).unwrap();
        if (tab.id() == "editor_filetab_selected") {
            before = tab.next_sibling();
            tab.remove_attribute("id").unwrap();
        }
        if (tab.get_attribute("editor_filetab_file_path").unwrap() == path) {
            found = Some(tab);
        }
    }
    found.unwrap_or_else(|| {
        let div = document.create_element("div").unwrap();
        div.class_list().toggle_with_force("hbox", true).unwrap();
        div.set_attribute("editor_filetab_file_id", &id.to_string()).unwrap();
        div.set_attribute("editor_filetab_file_path", path).unwrap();

        let name = document.create_element("div").unwrap();
        name.class_list().toggle_with_force("editor_filetab_name", true).unwrap();
        name.set_inner_html(path.split("/").last().unwrap());
        let open_callback = Closure::<dyn FnMut() -> ()>::new(move || crate::state::open_file(id));
        name.add_event_listener_with_callback("click", open_callback.as_ref().unchecked_ref()).unwrap();
        open_callback.forget();

        let close = document.create_element("div").unwrap();
        close.class_list().toggle_with_force("editor_filetab_close", true).unwrap();
        close.set_inner_html("×");
        let close_callback = Closure::<dyn FnMut() -> ()>::new(move || crate::state::close_file(id));
        close.add_event_listener_with_callback("click", close_callback.as_ref().unchecked_ref()).unwrap();
        close_callback.forget();

        div.append_child(&name).unwrap();
        div.append_child(&close).unwrap();
        filetabs.insert_before(&div, before.as_ref()).unwrap();
        div
    }).set_id("editor_filetab_selected");

    // File path
    set_filepath(path);
}


pub fn close(path : &str) {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // File tab
    let     filetabs = document.get_element_by_id("editor_filetabs").unwrap();
    let     children = filetabs.children();
    let mut found    = None;
    for i in 0..children.length() {
        let tab = children.get_with_index(i).unwrap();
        if (tab.get_attribute("editor_filetab_file_path").unwrap() == path) {
            let id = tab.get_attribute("editor_filetab_file_id").unwrap().parse::<u32>().unwrap();
            crate::code::destroy_monaco(id);
            if (tab.id() == "editor_filetab_selected") {
                tab.remove_attribute("id").unwrap();
                found = Some(i.saturating_sub(1));
            }
            filetabs.remove_child(&tab).unwrap();
            break;
        }
    }
    match (found.map(|i| filetabs.children().get_with_index(i))) {
        Some(Some(tab)) => {
            tab.set_id("editor_filetab_selected");
            let id   = tab.get_attribute("editor_filetab_file_id").unwrap().parse::<u32>().unwrap();
            let path = tab.get_attribute("editor_filetab_file_path").unwrap();
            set_filepath(&path);
            crate::filetree::open(&path);
            if let Some(FilesEntry { kind : FilesEntryKind::File { is_open }, .. }) = crate::state::FILES.read().get(&id) {
                match (is_open) {
                    Some(Some(FileContents::Text(_))) => { crate::code::open_monaco(id); },
                    Some(Some(FileContents::NonText)) => { crate::code::open_nontext(); },
                    Some(None) => { crate::code::open_load(); },
                    None => { crate::code::open_noopen(); }
                }
            }
        },
        Some(None) => {
            clear_filepath();
            crate::filetree::close(path);
            crate::code::open_noopen();
        },
        None => { }
    }
}


fn clear_filepath() {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let filepath = document.get_element_by_id("editor_filepath").unwrap();
    filepath.set_inner_html("");
}


fn set_filepath(path : &str) {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    clear_filepath();
    let filepath = document.get_element_by_id("editor_filepath").unwrap();
    for part in path.split("/") {
        let div = document.create_element("div").unwrap();
        div.set_inner_html(part);
        filepath.append_child(&div).unwrap();
    }
}


pub fn overwrite(id : u32, path : &str, contents : &FileContents) {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // File tab
    let filetabs = document.get_element_by_id("editor_filetabs").unwrap();
    let children = filetabs.children();
    for i in 0..children.length() {
        let tab = children.get_with_index(i).unwrap();
        if (tab.get_attribute("editor_filetab_file_path").unwrap() == path) {
            if (tab.id() == "editor_filetab_selected") {
                match (contents) {
                    FileContents::NonText => { crate::code::open_nontext(); },
                    FileContents::Text(text) => { crate::code::create_monaco(id, text, true) },
                }
            } else {
                match (contents) {
                    FileContents::NonText => { },
                    FileContents::Text(text) => { crate::code::create_monaco(id, text, false) },
                }
            }
            break;
        }
    }
}

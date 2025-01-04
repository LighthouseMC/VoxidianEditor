    mod monaco;
pub mod diffsync;
pub mod remote_cursors;


pub fn open_noopen() { open("editor_right_main_noopen"); }

pub fn open_nontext() { open("editor_right_main_nontext"); }

pub fn open_load() { open("editor_right_main_loader"); }

pub fn create_monaco(id : u32, initial_script : &str, open : bool) {
    if (open) { close(); }
    monaco::create(id, initial_script.to_string(), "text".to_string(), open);
}
pub fn open_monaco(id : u32) {
    close();
    monaco::open(id);
}

fn open(selected : &str) {
    let window    = web_sys::window().unwrap();
    let document  = window.document().unwrap();
    let container = document.get_element_by_id("editor_right_main_container").unwrap();
    let children  = container.children();
    for i in 0..children.length() {
        let child = children.get_with_index(i).unwrap();
        child.class_list().toggle_with_force("editor_right_main_selected", child.id() == selected).unwrap();
    }
}


pub fn destroy_monaco(id : u32) {
    monaco::destroy(id);
}

fn close() {
    let window    = web_sys::window().unwrap();
    let document  = window.document().unwrap();
    let container = document.get_element_by_id("editor_right_main_container").unwrap();
    let children  = container.children();
    for i in 0..children.length() {
        let child = children.get_with_index(i).unwrap();
        child.class_list().toggle_with_force("editor_right_main_selected", false).unwrap();
    }
}

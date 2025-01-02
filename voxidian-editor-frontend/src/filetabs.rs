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
        div.set_attribute("editor_filetab_file_path", path).unwrap();
        div.set_inner_html(path.split("/").last().unwrap());

        let close = document.create_element("div").unwrap();
        close.class_list().toggle_with_force("editor_filetab_close", true).unwrap();
        close.set_inner_html("×");
        let close_callback = Closure::<dyn FnMut() -> ()>::new(move || crate::state::close_file(id));
        close.add_event_listener_with_callback("click", close_callback.as_ref().unchecked_ref()).unwrap();
        close_callback.forget();

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
            if (tab.id() == "editor_filetab_selected") {
                found = Some((i > 0).then(|| i - 1));
            }
            filetabs.remove_child(&tab).unwrap();
            break;
        }
    }
    match (found) {
        Some(Some(i)) => {
            let tab  = children.get_with_index(i).unwrap();
            tab.set_id("editor_filetab_selected");
            let path = tab.get_attribute("editor_filetab_file_path").unwrap();
            set_filepath(&path);
            crate::filetree::open(&path);
        },
        Some(None) => {
            clear_filepath();
            crate::filetree::close(path);
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


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn alert(message : &str);
}

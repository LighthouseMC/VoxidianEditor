use std::sync::Mutex;


static COVER_OPEN        : Mutex<bool> = Mutex::new(true);
static COVER_LOADER_OPEN : Mutex<bool> = Mutex::new(true);
static COVER_ERROR_OPEN  : Mutex<bool> = Mutex::new(false);


fn open_cover() {
    let mut cover_open = COVER_OPEN.lock().unwrap();
    if (! *cover_open) {
        let window   = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let cover    = document.get_element_by_id("cover").unwrap();
        cover.class_list().toggle_with_force("cover_open", true).unwrap();
        *cover_open = true;
    }
}

fn close_cover() {
    let mut cover_open = COVER_OPEN.lock().unwrap();
    if (*cover_open) {
        let loader_open = COVER_LOADER_OPEN .lock().unwrap();
        let error_open  = COVER_ERROR_OPEN  .lock().unwrap();
        if ((! *loader_open) && (! *error_open)) {
            let window   = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let cover    = document.get_element_by_id("cover").unwrap();
            cover.class_list().toggle_with_force("cover_open", false).unwrap();
            *cover_open = false;
        }
    }
}


pub fn open_cover_error(error_message : &str) {
    let mut error_open = COVER_ERROR_OPEN.lock().unwrap();
    if (! *error_open) {
        open_cover();
        let window      = web_sys::window().unwrap();
        let document    = window.document().unwrap();
        let cover_error = document.get_element_by_id("cover_error").unwrap();
        cover_error.class_list().toggle_with_force("cover_open", true).unwrap();
        let cover_error_message = document.get_element_by_id("cover_error_message").unwrap();
        cover_error_message.set_inner_html(error_message);
        *error_open = true;
    }
}


pub fn close_cover_loader() {
    let mut loader_open = COVER_LOADER_OPEN.lock().unwrap();
    if (*loader_open) {
        let window       = web_sys::window().unwrap();
        let document     = window.document().unwrap();
        let cover_loader = document.get_element_by_id("cover_loader").unwrap();
        cover_loader.class_list().toggle_with_force("cover_open", false).unwrap();
        *loader_open = false;
        drop(loader_open);
        close_cover();
    }
}

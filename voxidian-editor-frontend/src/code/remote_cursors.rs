pub fn init_css() {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let     head  = document.get_element_by_id("head").unwrap();
    let     style = document.create_element("style").unwrap();
    let mut inner = String::new();

    for hue in 0..360 {
        inner += &format!(".editor_code_remote_selection_{}{{background:hsla({},100%,62.5%,0.875);min-width:2px;}}", hue, hue);
    }

    style.set_inner_html(&inner);
    head.append_child(&style).unwrap();
}

use wasm_bindgen::prelude::*;
use web_sys::Element;
use js_sys::Array;
use serde::Serialize as Ser;


mod js { use super::*;

    #[wasm_bindgen]
    extern "C" {

        #[wasm_bindgen(js_namespace = require)]
        pub(super) fn config(config : &JsValue);

        pub(super) fn require(from : &JsValue, callback : &JsValue);

        #[wasm_bindgen(js_namespace = ["monaco", "editor"], js_name = "onDidCreateEditor")]
        pub(super) fn on_did_create_editor(callback : &JsValue);

    }

    #[wasm_bindgen]
    extern "C" {

        #[wasm_bindgen(js_namespace = ["monaco", "editor"], js_name = "create")]
        pub(super) fn editor_create(on: &Element, config : &JsValue) -> Editor;
    }

    #[wasm_bindgen]
    extern "C" {
        pub type Editor;

        #[wasm_bindgen(method)]
        pub(super) fn layout(this : &Editor);
    }

}


#[derive(Ser)]
struct MonacoConfig {
    paths : MonacoConfigPaths
}
#[derive(Ser)]
struct MonacoConfigPaths {
    vs : String
}

#[derive(Ser)]
struct EditorConfig {
    value                     : String,
    language                  : String,
    theme                     : String,
    #[serde(rename = "autoDetectHighContrast")]
    auto_detect_high_contrast : bool,
    #[serde(rename = "automaticLayout")]
    automatic_layout          : bool,
    #[serde(rename = "cursorBlinking")]
    cursor_blinking           : String,
    #[serde(rename = "fontFamily")]
    font_family               : String,
    #[serde(rename = "fontLigatures")]
    font_ligatures            : bool,
    #[serde(rename = "fontSize")]
    font_size                 : f32,
    #[serde(rename = "fontWeight")]
    font_weight               : String,
    minimap                   : EditorConfigMinimap,
    #[serde(rename = "renderFinalNewline")]
    render_final_newline      : String,
    #[serde(rename = "smoothScrolling")]
    smooth_scrolling          : bool
}
#[derive(Ser)]
struct EditorConfigMinimap {
    #[serde(rename = "showSlider")]
    show_slider : String,
    size        : String
}


pub fn create(id : u32, initial_script : String, initial_language : String, open : bool) {
    require(move || {
        let window   = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let container = document.create_element("div").unwrap();
        container.class_list().toggle_with_force("editor_code_container", true).unwrap();
        container.set_attribute("editor_code_file_id", &id.to_string()).unwrap();

        let code = document.create_element("div").unwrap();
        code.class_list().toggle_with_force("editor_code", true).unwrap();
        container.append_child(&code).unwrap();

        document.get_element_by_id("editor_right_main_container").unwrap().append_child(&container).unwrap();
        if (open) {
            container.class_list().toggle_with_force("editor_right_main_selected", true).unwrap();
        }

        let config = EditorConfig {
            value                     : initial_script.clone(),
            language                  : initial_language.clone(),
            theme                     : "hc-black".to_string(),
            auto_detect_high_contrast : false,
            automatic_layout          : true,
            cursor_blinking           : "smooth".to_string(),
            font_family               : "Fira Code".to_string(),
            font_ligatures            : true,
            font_size                 : 13.0,
            font_weight               : "350".to_string(),
            minimap                   : EditorConfigMinimap {
                show_slider : "always".to_string(),
                size        : "proportional".to_string()
            },
            render_final_newline      : "dimmed".to_string(),
            smooth_scrolling          : true
        };
        js::editor_create(&code, &serde_wasm_bindgen::to_value(&config).unwrap());
        // TODO: events
    });
}


pub fn open(id : u32) {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let containers = document.get_elements_by_class_name("editor_code_container");
    let id = id.to_string();
    for i in 0..containers.length() {
        let container = containers.get_with_index(i).unwrap();
        if (container.get_attribute("editor_code_file_id").unwrap() == id) {
            container.class_list().toggle_with_force("editor_right_main_selected", true).unwrap();
        }
    }
}


pub fn destroy(id : u32) {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let containers = document.get_elements_by_class_name("editor_code_container");
    let id = id.to_string();
    for i in 0..containers.length() {
        let container = containers.get_with_index(i).unwrap();
        if (container.get_attribute("editor_code_file_id").unwrap() == id) {
            document.get_element_by_id("editor_right_main_container").unwrap().remove_child(&container).unwrap();
            break;
        }
    }
}


fn require<F : Fn() -> () + 'static>(f : F) {
    let config = MonacoConfig { paths : MonacoConfigPaths { vs : "https://unpkg.com/monaco-editor@latest/min/vs".to_string() } };
    js::config(&serde_wasm_bindgen::to_value(&config).unwrap());

    let from = Array::new();
    from.push(&JsValue::from_str("vs/editor/editor.main"));
    let callback = Closure::<dyn FnMut() -> ()>::new(move || f());
    js::require(from.unchecked_ref(), callback.as_ref().unchecked_ref());
    callback.forget();
}

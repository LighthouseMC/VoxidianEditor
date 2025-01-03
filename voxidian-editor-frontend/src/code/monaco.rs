use std::cell::LazyCell;
use std::sync::{ RwLock, RwLockReadGuard, RwLockWriteGuard };
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::Element;
use js_sys::Array;
use serde::Serialize as Ser;


pub(super) static EDITORS : EditorsContainer = EditorsContainer::new();
pub(super) struct EditorsContainer {
    files : LazyCell<RwLock<HashMap<u32, js::Editor>>>
}
impl EditorsContainer { const fn new() -> Self { Self {
    files : LazyCell::new(|| RwLock::new(HashMap::new()))
} } }
impl EditorsContainer {
    pub(super) fn read(&self) -> RwLockReadGuard<HashMap<u32, js::Editor>> {
        self.files.read().unwrap()
    }
    pub(super) fn write(&self) -> RwLockWriteGuard<HashMap<u32, js::Editor>> {
        self.files.write().unwrap()
    }
}
unsafe impl Sync for EditorsContainer { }


mod js { use super::*;

    #[wasm_bindgen]
    extern "C" {

        #[wasm_bindgen(js_namespace = require)]
        pub(super) fn config(config : &JsValue);

        pub(super) fn require(from : &JsValue, callback : &JsValue);

        /// https://microsoft.github.io/monaco-editor/docs.html#interfaces/editor.IStandaloneCodeEditor.html#getModel.getModel-1
        #[wasm_bindgen(js_namespace = ["monaco", "editor"], js_name = "onDidCreateEditor")]
        pub(super) fn on_did_create_editor(callback : &JsValue);

    }

    #[wasm_bindgen]
    extern "C" {

        /// https://microsoft.github.io/monaco-editor/docs.html#functions/editor.create.html
        #[wasm_bindgen(js_namespace = ["monaco", "editor"], js_name = "create")]
        pub(super) fn editor_create(on: &Element, config : &JsValue) -> Editor;
    }

    #[wasm_bindgen]
    extern "C" {
        pub type Editor;

        /// https://microsoft.github.io/monaco-editor/docs.html#interfaces/editor.IStandaloneCodeEditor.html#getSelections.getSelections-1
        #[wasm_bindgen(method, js_name = "getSelections")]
        pub fn get_selections(this : &Editor) -> Vec<EditorSelection>;

        /// https://microsoft.github.io/monaco-editor/docs.html#interfaces/editor.IStandaloneCodeEditor.html
        #[wasm_bindgen(method, js_name = "setSelections")]
        pub fn set_selections(this : &Editor, selections : Vec<EditorSelection>);

        /// https://microsoft.github.io/monaco-editor/docs.html#interfaces/editor.IStandaloneCodeEditor.html#getModel.getModel-1
        #[wasm_bindgen(method, js_name = "getModel")]
        pub fn get_model(this : &Editor) -> EditorModel;
    }

    #[wasm_bindgen]
    extern "C" {
        pub type EditorSelection;

        #[wasm_bindgen(constructor)]
        pub fn new_selection(start_line : usize, start_column : usize, end_line : usize, end_column : usize) -> EditorSelection;

        /// https://microsoft.github.io/monaco-editor/typedoc/classes/Selection.html#startLineNumber
        #[wasm_bindgen(getter, js_name = "startLineNumber")]
        pub fn get_start_line(this : &EditorSelection) -> usize;

        /// https://microsoft.github.io/monaco-editor/typedoc/classes/Selection.html#startColumn
        #[wasm_bindgen(getter, js_name = "startColumn")]
        pub fn get_start_column(this : &EditorSelection) -> usize;

        /// https://microsoft.github.io/monaco-editor/docs.html#classes/Selection.html#endLineNumber
        #[wasm_bindgen(getter, js_name = "endLineNumber")]
        pub fn get_end_line(this : &EditorSelection) -> usize;

        /// https://microsoft.github.io/monaco-editor/typedoc/classes/Selection.html#endColumn
        #[wasm_bindgen(getter, js_name = "endColumn")]
        pub fn get_end_column(this : &EditorSelection) -> usize;

    }

    #[wasm_bindgen]
    extern "C" {
        pub type EditorModel;

        /// https://microsoft.github.io/monaco-editor/docs.html#interfaces/editor.ITextModel.html#getValue.getValue-1
        #[wasm_bindgen(method, js_name = "getValue")]
        pub fn get_value(this : &EditorModel, eol_preference : u8) -> String;

        /// https://microsoft.github.io/monaco-editor/docs.html#interfaces/editor.ITextModel.html#setValue.setValue-1
        #[wasm_bindgen(method, js_name = "setValue")]
        pub fn set_value(this : &EditorModel, value : &str);

        /// https://microsoft.github.io/monaco-editor/typedoc/interfaces/editor.ITextModel.html#getOffsetAt.getOffsetAt-1
        #[wasm_bindgen(method, js_name = "getOffsetAt")]
        pub fn get_offset_at(this : &EditorModel, position : EditorPosition) -> usize;

        /// https://microsoft.github.io/monaco-editor/typedoc/interfaces/editor.ITextModel.html#getPositionAt.getPositionAt-1
        #[wasm_bindgen(method, js_name = "getPositionAt")]
        pub fn get_position_at(this : &EditorModel, offset : usize) -> EditorPosition;

    }

    #[wasm_bindgen]
    extern "C" {
        pub type EditorPosition;

        #[wasm_bindgen(constructor)]
        pub fn new_position(line : usize, column : usize) -> EditorPosition;

        #[wasm_bindgen(getter, js_name = "lineNumber")]
        pub fn line(this : &EditorPosition) -> usize;

        #[wasm_bindgen(getter, js_name = "column")]
        pub fn column(this : &EditorPosition) -> usize;

    }

}
pub use js::EditorSelection;
pub use js::EditorPosition;


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
        let editor = js::editor_create(&code, &serde_wasm_bindgen::to_value(&config).unwrap());
        // TODO: events
        EDITORS.write().insert(id, editor);
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
    let id_string = id.to_string();
    for i in 0..containers.length() {
        let container = containers.get_with_index(i).unwrap();
        if (container.get_attribute("editor_code_file_id").unwrap() == id_string) {
            document.get_element_by_id("editor_right_main_container").unwrap().remove_child(&container).unwrap();
            break;
        }
    }

    EDITORS.write().remove(&id);
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

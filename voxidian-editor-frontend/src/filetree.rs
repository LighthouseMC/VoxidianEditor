use crate::iter::IteratorExt;
use std::cell::LazyCell;
use std::sync::{ Mutex, MutexGuard };
use std::collections::HashMap;
use std::cmp::Ordering;
use voxidian_editor_common::packet::s2c::FileTreeEntry;
use wasm_bindgen::prelude::*;
use web_sys::Element;


static FILETREE : FileTreeRootContainer = FileTreeRootContainer::new();
struct FileTreeRootContainer {
    root  : LazyCell<Element>,
    nodes : LazyCell<Mutex<HashMap<String, Element>>>
}
impl FileTreeRootContainer { const fn new() -> Self { Self {
    root : LazyCell::new(|| {
        let window   = web_sys::window().unwrap();
        let document = window.document().unwrap();
        document.get_element_by_id("editor_filetree_root").unwrap()
    }),
    nodes : LazyCell::new(|| Mutex::new(HashMap::new()))
} } }
impl FileTreeRootContainer {

    fn root(&self) -> &Element { &self.root }

    fn nodes(&self) -> MutexGuard<HashMap<String, Element>> { self.nodes.lock().unwrap() }

}
unsafe impl Sync for FileTreeRootContainer { }


pub fn clear() {
    FILETREE.root().set_inner_html("");
    FILETREE.nodes().clear();
}


#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = console)]
    fn warn(message : &str);
}


pub fn add(entry : FileTreeEntry) {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let entry_root = document.create_element("li").unwrap();

    let filename = entry.path.split("/").last().unwrap();
    entry_root.set_attribute("editor_filetree_filename_lowercase", &filename.to_lowercase()).unwrap();

    if (entry.is_dir) {
        entry_root.set_attribute("editor_filetree_is_dir", "true").unwrap();

        let fold = document.create_element("div").unwrap();
        fold.class_list().toggle_with_force("editor_filetree_fold", true).unwrap();
        fold.set_inner_html(filename);
        entry_root.append_child(&fold).unwrap();

        let nest = document.create_element("ul").unwrap();
        nest.class_list().toggle_with_force("editor_filetree_nest", true).unwrap();
        entry_root.append_child(&nest).unwrap();

        let fold1 = fold.clone();
        let click_callback = Closure::<dyn FnMut() -> ()>::new(move || {
            fold1.class_list().toggle("editor_filetree_unfolded").unwrap();
            nest.class_list().toggle("editor_filetree_nest_unfolded").unwrap();
        });
        fold.add_event_listener_with_callback("click", click_callback.as_ref().unchecked_ref()).unwrap();
        click_callback.forget();

        // Add existing children
        let children_path_prefix = format!("{}/", entry.path);
        for (path, child) in &*FILETREE.nodes() {
            if (path.starts_with(&children_path_prefix)) {
                entry_root.query_selector(".editor_filetree_nest").unwrap().unwrap().append_child(child).unwrap();
            }
        }

    } else {

        let div = document.create_element("div").unwrap();
        div.set_inner_html(&format!("<i class=\"devicon-bash-plain noicon\"></i> {}", filename));
        let icon_classlist = div.query_selector("i").unwrap().unwrap().class_list();
        match (filename_to_icon(filename)) {
            None => {
                icon_classlist.toggle_with_force("noicon", true).unwrap();
                icon_classlist.toggle_with_force("devicon-bash-plain", true).unwrap();
            },
            Some((icon, coloured)) => {
                icon_classlist.toggle_with_force("icon", true).unwrap();
                icon_classlist.toggle_with_force(icon, true).unwrap();
                if (coloured) {
                    icon_classlist.toggle_with_force("colored", true).unwrap();
                }
            }
        }
        entry_root.append_child(&div).unwrap();

    }

    // Add to parent
    let parent_path = entry.path.split("/").skip_last().intersperse("/").collect::<String>();
    if (parent_path.is_empty()) {
        FILETREE.root().append_child(&entry_root).unwrap();
    } else if let Some(parent) = FILETREE.nodes().get(&parent_path) {
        parent.query_selector(".editor_filetree_nest").unwrap().unwrap().append_child(&entry_root).unwrap();
    }

    FILETREE.nodes().insert(entry.path, entry_root);
}


pub fn sort() {
    fn sort_one(entry_root : &Element) {
        let mut children = {
            let children = entry_root.children();
            let mut out = Vec::with_capacity(children.length() as usize);
            for i in 0..children.length() {
                out.push(children.get_with_index(i).unwrap());
            }
            out
        };
        children.sort_by(|a, b| {
            let a_is_dir = a.has_attribute("editor_filetree_is_dir");
            let b_is_dir = b.has_attribute("editor_filetree_is_dir");
            match ((a_is_dir, b_is_dir)) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                (true, true) | (false, false) => {
                    let a_filename_lowercase = a.get_attribute("editor_filetree_filename_lowercase").unwrap();
                    let b_filename_lowercase = b.get_attribute("editor_filetree_filename_lowercase").unwrap();
                    a_filename_lowercase.cmp(&b_filename_lowercase)
                }
            }
        });
        for child in children {
            entry_root.append_child(&child).unwrap();
        }
    }

    sort_one(FILETREE.root());
    for (_, entry_root) in &*FILETREE.nodes() {
        if let Some(entry_root) = entry_root.query_selector(".editor_filetree_nest").unwrap() {
            sort_one(&entry_root);
        }
    }
}


fn filename_to_icon(filename : &str) -> Option<(&'static str, bool)> {
    if let Some(split) = filename.chars().position(|ch| ch == '.') {
        let (_, ext) = filename.split_at(split + 1);
        Some(match (ext) {
            // Bash
            "sh" => ("devicon-bash-plain", false),
            // C
            "c" | "cats" | "h" | "idc" | "w"
                => ("devicon-c-plain", true),
            // C++
            "cpp" | "cc" | "cp" | "cxx" | "hh" | "hpp" | "hxx" | "inc" | "inl" | "ipp" | "tcc" | "tpp"
                => ("devicon-cplusplus-plain", true),
            // Rust
            "rs" => ("devicon-rust-original", false),
            // Go
            "go" => ("devicon-go-original-wordmark", true),
            // C#
            "cs" | "cake" | "cshtml" | "csx"
                => ("devicon-csharp-plain", true),
            // Dart
            "dart" => ("devicon-dart-plain", true),
            // F#
            "fs" | "fsi" | "fsx"
                => ("devicon-fsharp-plain", true),
            // LabVIEW
            "lvproj" => ("devicon-labview-plain", true),
            // Lua
            "lua" | "fcgi" | "nse" | "pd_lua" | "rbxs" | "wlua"
                => ("devicon-lua-plain", true),
            // TypeScript
            "ts" | "tsx"
                => ("devicon-typescript-plain", true),
            // WebAssembly
            "wasm" | "wat"
                => ("devicon-wasm-original", true),
            // Zig
            "zig" => ("devicon-zig-original", true),
            // Crystal
            "cr" => ("devicon-crystal-original", true),
            // Elixir
            "ex" | "exs"
                => ("devicon-elixir-plain", true),
            // Java
            "java" | "class"
                => ("devicon-java-plain", true),
            // Javascript
            "js" | "_js" | "bones" | "es" | "es6" | "frag" | "gs" | "jake" | "jsb" | "jscad" | "jsfl" | "jsm" | "jss" | "njs" | "pac" | "sjs" | "ssjs" | "sublime-build" | "sublime-commands" | "sublime-completions" | "sublime-keymap" | "sublime-macro" | "sublime-menu" | "sublime-mousemap" | "sublime-project" | "sublime-settings" | "sublime-theme" | "sublime-workspace" | "sublime_metrics" | "sublime_session" | "xsjs" | "xsjslib"
                => ("devicon-javascript-plain", true),
            // Kotlin
            "kt" | "ktm" | "kts"
                => ("devicon-kotlin-plain", true),
            // Perl
            "pl" | "al" | "perl" | "ph" | "plx" | "pm" | "pod" | "psgi" | "t" | "6pl" | "6pm" | "nqp" | "p6" | "p6l" | "p6m" | "pl6" | "pm6"
                => ("devicon-perl-plain", true),
            // PHP
            "php" | "aw" | "ctp" | "inc" | "php3" | "php4" | "php5" | "phps" | "phpt"
                => ("devicon-php-plain", true),
            // Python
            "py" | "bzl" | "cgi" | "fcgi" | "gyp" | "lmi" | "pyde" | "pyp" | "pyt" | "pyw" | "rpy" | "tac" | "wsgi" | "xpy"
                => ("devicon-python-plain", true),
            // Prolog
            "pro" | "prolog" | "yap"
                => ("devicon-prolog-plain", true),
            // R
            "r" | "rd" | "rsx"
                => ("devicon-r-plain", true),
            // Ruby
            "rb" | "builder" | "fcgi" | "gemspec" | "god" | "irbrc" | "jbuilder" | "mspec" | "pluginspec" | "podspec" | "rabl" | "rake" | "rbuild" | "rbw" | "rbx" | "ru" | "ruby" | "thor" | "watchr"
                => ("devicon-ruby-plain", true),
            // Swift
            "swift" => ("devicon-swift-plain", true),
            // Ballerina
            "bal" => ("devicon-ballerina-original", true),
            // Haskell
            "hs" | "hsc"
                => ("devicon-haskell-plain", true),
            // Julia
            "hl" => ("devicon-julia-plain", true),
            // Nim
            "nim" | "nimrod"
                => ("devicon-nim-plain", true),
            // OCaml
            "ml" | "eliom" | "eliomi" | "ml4" | "mli" | "mll" | "mly"
                => ("devicon-ocaml-plain", true),

            _ => { return None }
        })
    } else { None }
}

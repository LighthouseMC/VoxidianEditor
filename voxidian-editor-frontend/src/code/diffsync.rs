//! https://neil.fraser.name/writing/sync/


use crate::state::FilesEntryKind;
use super::monaco::{ self, EditorPosition, EditorSelection, EditorSetSelection };
use voxidian_editor_common::packet::s2c::FileContents;
use voxidian_editor_common::packet::c2s::PatchFileC2SPacket;
use voxidian_editor_common::dmp::{ DiffMatchPatch, Efficient, PatchInput, Patches };


pub fn send_patches_to_server() {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let containers = document.get_elements_by_class_name("editor_code_container");
    for i in 0..containers.length() {
        let container = containers.get_with_index(i).unwrap();
        let id = container.get_attribute("editor_code_file_id").unwrap().parse::<u32>().unwrap();

        let mut files = crate::state::FILES.write();
        let FilesEntryKind::File { is_open : Some(Some(FileContents::Text(client_shadow))) } = &mut files.get_mut(&id).unwrap().kind else { continue };
        let client_text = monaco::EDITORS.read()[&id].get_model().get_value(1);

        if (client_shadow != &client_text) {
            let dmp = DiffMatchPatch::new();
            // Client Text is diffed against the Client Shadow.
            let diffs = dmp.diff_main::<Efficient>(client_shadow, &client_text).unwrap();
            // This returns a list of edits which have been performed on Client Text.
            let patches = dmp.patch_make(PatchInput::new_diffs(&diffs)).unwrap();
            // Client Text is copied over to Client Shadow.
            *client_shadow = client_text;
            // The edits are sent to the Server.
            crate::ws::WS.send(PatchFileC2SPacket {
                id,
                patches
            });
        }
    }
}


pub fn apply_patches_from_server(id : u32, patches : Patches<Efficient>) {
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let containers = document.get_elements_by_class_name("editor_code_container");
    let id_string = id.to_string();
    for i in 0..containers.length() {
        let container = containers.get_with_index(i).unwrap();
        if (container.get_attribute("editor_code_file_id").unwrap() == id_string) {
            let client_editor   = &monaco::EDITORS.read()[&id];
            let client_model    = client_editor.get_model();
            let old_client_text = client_model.get_value(1);
            let mut selections  = client_editor.get_selections().into_iter().map(|s| {
                let s = serde_wasm_bindgen::from_value::<EditorSelection>(s).unwrap();
                Selection {
                    start : client_model.get_offset_at(serde_wasm_bindgen::to_value(&EditorPosition { line : s.start_line, column : s.start_column }).unwrap()),
                    end   : client_model.get_offset_at(serde_wasm_bindgen::to_value(&EditorPosition { line : s.end_line,   column : s.end_column   }).unwrap())
                }
            }).collect::<Vec<_>>();

            let dmp = DiffMatchPatch::new();
            let (new_client_text, _) = dmp.patch_apply(&patches, &old_client_text).unwrap();
            for selection in &mut selections {
                selection.start = check_cursor(&old_client_text, &new_client_text, selection.start);
                selection.end   = check_cursor(&old_client_text, &new_client_text, selection.end);
            }

            client_model.set_value(&new_client_text);
            
            client_editor.set_selections(selections.into_iter().map(|s| {
                let start = serde_wasm_bindgen::from_value::<EditorPosition>(client_model.get_position_at(s.start )).unwrap();
                let end   = serde_wasm_bindgen::from_value::<EditorPosition>(client_model.get_position_at(s.end   )).unwrap();
                serde_wasm_bindgen::to_value(&EditorSetSelection {
                    start_line   : start.line,
                    start_column : start.column,
                    end_line     : end.line,
                    end_column   : end.column
                }).unwrap()
            }).collect::<Vec<_>>());
            break;
        }
    }
}

fn check_cursor(old_client_text : &str, new_client_text : &str, index : usize) -> usize {
    let old_slice = old_client_text.chars().enumerate().filter_map(|(i, ch)| (i < index).then(|| ch)).collect::<Vec<_>>();
    let new_slice = new_client_text.chars().enumerate().filter_map(|(i, ch)| (i < index).then(|| ch)).collect::<Vec<_>>();
    let is_front = old_slice != new_slice;
    if (is_front) {
        let diff = (old_client_text.chars().count() as isize) - (new_client_text.chars().count() as isize);
        index.saturating_sub_signed(diff)
    } else { index }
}

pub struct Selection {
    start : usize,
    end   : usize
}

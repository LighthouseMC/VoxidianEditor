//! https://neil.fraser.name/writing/sync/


use crate::state::FilesEntryKind;
use super::monaco;
use voxidian_editor_common::packet::s2c::FileContents;
use voxidian_editor_common::packet::c2s::PatchFileC2SPacket;
use voxidian_editor_common::dmp::{ DiffMatchPatch, Patches, PatchInput, Efficient };


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
            let client_editor = &monaco::EDITORS.read()[&id];
            let client_model  = client_editor.get_model();
            let client_text   = client_model.get_value(1);
            let selections    = client_editor.get_selections();

            let dmp = DiffMatchPatch::new();
            let (client_text, _) = dmp.patch_apply(&patches, &client_text).unwrap();

            client_model.set_value(&client_text);
            client_editor.set_selections(selections);
            break;
        }
    }
}

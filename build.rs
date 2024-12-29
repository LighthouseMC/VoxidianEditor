#![feature(exit_status_error)]


use std::io::Write;
use std::process::Command;
use std::env::current_dir;
use std::fs::{ File, read_to_string, remove_file };
use std::path;
use walkdir::WalkDir;


fn main() {
    // Skip this if it's rust-analyser running.
    if let Some(_) = option_env!("RUSTANALYSER") { return; }

    // Get paths.
    /*let cwd       = path::absolute(current_dir().unwrap()).unwrap();
    let wasm_dir  = cwd.join("src").join("assets").join("code");
    let wasm_file = wasm_dir.join("voxidian-editor_bg.wasm");
    println!("cargo::rerun-if-changed={}", wasm_file.display());
    let js_file   = wasm_dir.join("voxidian-editor.js");
    println!("cargo::rerun-if-changed={}", js_file.display());

    // Build `voxidian-editor-frontend`.
    Command::new("cargo")
        .args([
            "build",
            "--target=wasm32-unknown-unknown",
            "--release"
        ])
        .current_dir("voxidian-editor-frontend")
        .status().unwrap()
        .exit_ok().unwrap();


    // Install `wasm-bindgen` if it isn't installed already.
    Command::new("cargo")
        .args(["install", "wasm-bindgen-cli"])
        .status().unwrap()
        .exit_ok().unwrap();

    // Remove old files.
    let _ = remove_file(&wasm_file);
    let _ = remove_file(&js_file);

    // Generate bindings.
    Command::new("wasm-bindgen")
        .args([
            "--no-typescript",
            "--target", "web",
            "--out-dir", "src/assets/code",
            "--out-name", "voxidian-editor",
            "voxidian-editor-frontend/target/wasm32-unknown-unknown/release/voxidian-editor-frontend.wasm"
        ])
        .status().unwrap()
        .exit_ok().unwrap();

    // Change WASM file request route in JS file.
    let script = read_to_string(&js_file).unwrap().replace("voxidian-editor_bg.wasm", "/assets/code/editor.wasm");
    let mut f = File::create(&js_file).unwrap();
    write!(f, "{}", script).unwrap();

    // Rebuild if any file in `voxidian-editor-frontend` has changed.
    for entry in WalkDir::new(cwd.join("voxidian-editor-frontend")) {
        if let Ok(entry) = entry {
            println!("cargo::rerun-if-changed={}", entry.path().display());
        }
    }*/

}

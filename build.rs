#![feature(exit_status_error)]


use std::process::Command;
use walkdir::WalkDir;


fn main() {

    let voxidian_editor_commit    = String::from_utf8(Command::new("git").args(["rev-parse", "HEAD"]).output().unwrap().stdout).unwrap().replace("\n", "");
    let voxidian_editor_timestamp = String::from_utf8(Command::new("git").args(["show", "-s", "--format=%ci", "HEAD"]).output().unwrap().stdout).unwrap().replace("\n", "");
    println!("cargo:rustc-env=VOXIDIAN_EDITOR_COMMIT={} {}", &voxidian_editor_commit[..9], voxidian_editor_timestamp.split(" ").nth(0).unwrap());
    println!("cargo:rustc-env=VOXIDIAN_EDITOR_COMMIT_HASH={}", voxidian_editor_commit);

    Command::new("wasm-pack").args(["build", "--target", "web"]).current_dir("voxidian-editor-frontend").status().unwrap().exit_ok().unwrap();
    for entry in WalkDir::new("voxidian-editor-frontend") {
        if let Ok(entry) = entry {
            println!("cargo::rerun-if-changed={}", entry.path().display());
        }
    }
    for entry in WalkDir::new("voxidian-editor-common") {
        if let Ok(entry) = entry {
            println!("cargo::rerun-if-changed={}", entry.path().display());
        }
    }

}

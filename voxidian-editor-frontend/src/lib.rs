#![feature(
    sync_unsafe_cell,
    iter_intersperse
)]


// Utility
mod iter;

// Connection
mod ws;
mod state;

// UX
mod cover;
mod filetree;
mod filetabs;


use wasm_bindgen::prelude::*;


#[wasm_bindgen(start)]
pub fn start() {
    ws::start();
}

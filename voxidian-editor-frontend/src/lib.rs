#![feature(
    sync_unsafe_cell,
    iter_intersperse
)]


mod iter;

mod cover;
mod ws;
mod filetree;


use wasm_bindgen::prelude::*;


#[wasm_bindgen(start)]
pub fn start() {
    ws::start();
}

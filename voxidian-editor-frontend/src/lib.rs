#![feature(
    sync_unsafe_cell,
    iter_intersperse,
    mixed_integer_ops_unsigned_sub
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
mod code;


use wasm_bindgen::prelude::*;


#[wasm_bindgen(start)]
pub fn start() {
    ws::start();
}


// TODO: Remove
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn alert(message : &str);
    #[wasm_bindgen(js_namespace = console)]
    fn warn(msg : &str);
}

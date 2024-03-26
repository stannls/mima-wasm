#![feature(const_refs_to_static)]

use wasm_bindgen::prelude::*;
mod mima;

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello {}", name));
}

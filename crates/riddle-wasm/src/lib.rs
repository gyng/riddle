//! Thin wasm-bindgen bridge over riddle-core. JSON strings in and out. See docs/CUT1.md.
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn version() -> String { "0.1.0".into() }

use wasm_bindgen::prelude::*;

/// Convert an HTML document (with optional CSS) into PDF bytes.
#[wasm_bindgen]
pub fn convert(html: &str, css: Option<String>) -> Vec<u8> {
    docoxide::convert(html, css.as_deref())
}

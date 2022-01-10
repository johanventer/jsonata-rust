use wasm_bindgen::prelude::*;

use jsonata::JsonAta;

#[wasm_bindgen]
pub fn evaluate(expr: &str, input: &str) -> String {
    let jsonata = JsonAta::new(expr).unwrap();
    jsonata
        .evaluate(Some(input))
        .map(|result| result.to_string())
        .unwrap_or_else(|e| e.to_string())
}

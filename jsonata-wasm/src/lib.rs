use wasm_bindgen::prelude::*;

use jsonata::{JsonAta, Value};

#[wasm_bindgen]
pub fn evaluate(expr: &str, input: &str) -> Result<JsValue, JsValue> {
    let jsonata = JsonAta::new(expr).map_err(|e| JsValue::from(e.to_string()))?;
    jsonata
        .evaluate(Some(input))
        .map(|result| match result.as_ref(&jsonata.arena) {
            Value::Undefined => JsValue::UNDEFINED,
            _ => JsValue::from(result.as_ref(&jsonata.arena).dump()),
        })
        .map_err(|e| JsValue::from(e.to_string()))
}

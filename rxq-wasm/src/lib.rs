mod document;
mod format;
mod query;

pub use document::RxqDocument;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
}

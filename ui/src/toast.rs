use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub fn displaySuccess(message: String);
    pub fn displayInfo(message: String);
    pub fn displayError(message: String);
}

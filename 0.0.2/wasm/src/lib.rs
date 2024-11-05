use crate::prelude::*;

mod utils;
mod appnote63;
mod context;
mod prelude;
mod alert;

#[wasm_bindgen]
pub fn main() {
    context::Context::init();
}

use crate::prelude::*;

mod utils;
mod appnote63;
mod context;
mod prelude;
mod alert;
mod constant;

#[wasm_bindgen]
pub fn main() {
    utils::set_panic_hook();
    context::Context::init();
}

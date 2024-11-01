use web_sys::js_sys::Uint8Array;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub struct Uint8ArrayWriter<'a> {
    array: &'a Uint8Array,
    pos: u32,
}

impl <'a> Uint8ArrayWriter<'a> {
    pub fn new(array: &'a Uint8Array) -> Self {
        Self {
            array,
            pos: 0,
        }
    }

    pub fn write_slice(&mut self, slice: &[u8]) -> u32 {
        self.array.subarray(self.pos, self.pos + slice.len() as u32).copy_from(slice);
        self.pos += slice.len() as u32;
        slice.len() as u32
    }
}

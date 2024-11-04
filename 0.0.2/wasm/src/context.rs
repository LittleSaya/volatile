//! A `Context` is similar to a global variable, which will be initialized only once, captured by every closure, then forgot in memory.

use std::{ cell::{OnceCell, RefCell}, rc::Rc };

use web_sys::{ js_sys, wasm_bindgen };
use wasm_bindgen::prelude::*;

use crate::appnote63;

mod create_context;
mod callback;
mod rust_closure;
mod event_handler;

pub struct Context {
  pub window                 : Rc<web_sys::Window>,
  pub element                : Rc<ContextElement>,
  pub scan_stage             : Rc<ContextScanStage>,
  pub compress_encrypt_stage : Rc<ContextCompressEncryptStage>,
  pub callback               : Rc<ContextCallback>,
  pub rust_closure           : Rc<ContextRustClosure>,
  pub event_handler          : Rc<ContextEventHandler>,
}

pub struct ContextElement {
  pub dropping_area    : Rc<web_sys::HtmlDivElement>,
  pub status           : Rc<web_sys::HtmlDivElement>,
  pub compress_encrypt : Rc<web_sys::HtmlButtonElement>,
  pub decrypt          : Rc<web_sys::HtmlButtonElement>,
}

pub struct ContextScanStage {
  pub file_system            : Rc<RefCell<Option<web_sys::FileSystem>>>,
  pub file_path_list         : Rc<RefCell<Vec<FilePath>>>,
  pub unresolved_directories : Rc<RefCell<u64>>,
}

pub struct FilePath {
  pub path   : String,
  pub is_dir : bool,
}

pub struct ContextCompressEncryptStage {
  pub writer            : Rc<RefCell<Option<web_sys::WritableStreamDefaultWriter>>>,
  pub number_compressed : Rc<RefCell<u64>>,
  pub file_headers      : Rc<RefCell<Vec<appnote63::FileHeader>>>,
  pub bytes_written     : Rc<RefCell<u64>>,
  pub buffer_header     : Rc<RefCell<js_sys::ArrayBuffer>>,
  pub buffer_data       : Rc<RefCell<js_sys::ArrayBuffer>>,
  pub buffer_data_wasm  : Rc<RefCell<Vec<u8>>>,
}

#[allow(non_snake_case)]
pub struct ContextCallback {
  pub FileSystemFileEntry_file_success              : Rc<OnceCell<Closure<dyn Fn(web_sys::File)>>>,
  pub FileSystemFileEntry_file_error                : Rc<OnceCell<Closure<dyn Fn(web_sys::DomException)>>>,
  pub FileSystemDirectoryEntry_getFile_success      : Rc<OnceCell<Closure<dyn Fn(web_sys::FileSystemFileEntry)>>>,
  pub FileSystemDirectoryEntry_getFile_error        : Rc<OnceCell<Closure<dyn Fn(web_sys::DomException)>>>,
  pub FileSystemDirectoryReader_readEntries_success : Rc<OnceCell<Closure<dyn Fn(js_sys::Array)>>>,
  pub FileSystemDirectoryReader_readEntries_error   : Rc<OnceCell<Closure<dyn Fn(web_sys::DomException)>>>,
}

pub struct ContextRustClosure {
  pub scan              : Rc<OnceCell<Box<dyn Fn(Vec<web_sys::FileSystemEntry>)>>>,
  pub take_item         : Rc<OnceCell<Box<dyn Fn()>>>,
  pub get_file_entry    : Rc<OnceCell<Box<dyn Fn(String)>>>,
  pub process_directory : Rc<OnceCell<Box<dyn Fn(String)>>>,
}

#[allow(non_snake_case)]
pub struct ContextEventHandler {
  pub dropping_area__dragover : Rc<OnceCell<Closure<dyn Fn(web_sys::DragEvent)>>>,
  pub dropping_area__drop     : Rc<OnceCell<Closure<dyn Fn(web_sys::DragEvent)>>>,
  pub compress_encrypt__click : Rc<OnceCell<Closure<dyn Fn(web_sys::PointerEvent)>>>,
}

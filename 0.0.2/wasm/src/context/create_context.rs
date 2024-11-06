use std::{cell::{OnceCell, RefCell}, mem, rc::Rc};

use crate::{constant::{BUFFER_DATA_SIZE, BUFFER_HEADER_SIZE}, prelude::*};

use super::{callback, event_handler, rust_closure, Context, ContextCallback, ContextCompressEncryptStage, ContextElement, ContextEventHandler, ContextRustClosure, ContextScanStage};

impl Context {

  /// Initialize context
  pub fn init() {

    let window = web_sys::window().unwrap();

    let document = window.document().unwrap();

    let context = Rc::new(
      Self {
        window: Rc::new(window.clone()),
        performance: Rc::new(window.performance().unwrap()),

        element: Rc::new(ContextElement {
          dropping_area    : Rc::new( ensure_element::<web_sys::HtmlDivElement>(&window, &document, "dropping_area") ),
          status           : Rc::new( ensure_element::<web_sys::HtmlDivElement>(&window, &document, "status") ),
          compress_encrypt : Rc::new( ensure_element::<web_sys::HtmlButtonElement>(&window, &document, "compress_encrypt") ),
          decrypt          : Rc::new( ensure_element::<web_sys::HtmlButtonElement>(&window, &document, "decrypt") ),
        }),

        scan_stage: Rc::new(ContextScanStage {
          file_system            : Rc::new(RefCell::new(None)),
          file_path_list         : Rc::new(RefCell::new(Vec::new())),
          unresolved_directories : Rc::new(RefCell::new(0)),
        }),

        compress_encrypt_stage: Rc::new(ContextCompressEncryptStage {
          writer            : Rc::new(RefCell::new(None)),
          number_compressed : Rc::new(RefCell::new(0)),
          file_headers      : Rc::new(RefCell::new(Vec::new())),
          bytes_written     : Rc::new(RefCell::new(0)),
          buffer_header     : Rc::new(RefCell::new(Vec::with_capacity(BUFFER_HEADER_SIZE as usize))),
          buffer_data       : Rc::new(RefCell::new(vec![0; BUFFER_DATA_SIZE as usize])),
        }),

        callback: Rc::new(ContextCallback {
            FileSystemFileEntry_file_success              : Rc::new(OnceCell::new()),
            FileSystemFileEntry_file_error                : Rc::new(OnceCell::new()),
            FileSystemDirectoryEntry_getFile_success      : Rc::new(OnceCell::new()),
            FileSystemDirectoryEntry_getFile_error        : Rc::new(OnceCell::new()),
            FileSystemDirectoryReader_readEntries_success : Rc::new(OnceCell::new()),
            FileSystemDirectoryReader_readEntries_error   : Rc::new(OnceCell::new()),
        }),

        rust_closure: Rc::new(ContextRustClosure {
          scan              : Rc::new(OnceCell::new()),
          take_item         : Rc::new(OnceCell::new()),
          get_file_entry    : Rc::new(OnceCell::new()),
          process_directory : Rc::new(OnceCell::new()),
          finish            : Rc::new(OnceCell::new()),
        }),

        event_handler: Rc::new(ContextEventHandler {
          dropping_area__dragover : Rc::new(OnceCell::new()),
          dropping_area__drop     : Rc::new(OnceCell::new()),
          compress_encrypt__click : Rc::new(OnceCell::new()),
        }),
      }
    );

    callback::FileSystemFileEntry_file_success::init(&context);
    callback::FileSystemFileEntry_file_error::init(&context);
    callback::FileSystemDirectoryEntry_getFile_success::init(&context);
    callback::FileSystemDirectoryEntry_getFile_error::init(&context);
    callback::FileSystemDirectoryReader_readEntries_success::init(&context);
    callback::FileSystemDirectoryReader_readEntries_error::init(&context);

    rust_closure::scan::init(&context);
    rust_closure::take_item::init(&context);
    rust_closure::get_file_entry::init(&context);
    rust_closure::process_directory::init(&context);
    rust_closure::finish::init(&context);

    event_handler::dropping_area__dragover::init(&context);
    event_handler::dropping_area__drop::init(&context);
    event_handler::compress_encrypt__click::init(&context);

    mem::forget(context);
  }
}

/// Ensure the existence and type of a specific element.
fn ensure_element<T: JsCast>(window: &web_sys::Window, document: &web_sys::Document, element_id: &str) -> T {
  let Some(element) = document.get_element_by_id(&element_id) else {
    error(window, &format!("{} 元素不存在。", element_id));
  };

  let Ok(typed_element) = element.dyn_into::<T>() else {
    error(window, &format!("{} 元素类型异常", element_id));
  };

  typed_element
}

fn error(window: &web_sys::Window, msg: &str) -> ! {
  window.alert_with_message(msg).unwrap();
  panic!();
}

use std::rc::Rc;

use crate::{alert, prelude::*};

use super::super::Context;

/// Just register next callback.
pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.callback.FileSystemDirectoryEntry_getFile_success.set(Closure::<dyn Fn(web_sys::FileSystemFileEntry)>::new(move |file_entry: web_sys::FileSystemFileEntry| {
    let context = &context_clone;
    file_entry.file_with_callback_and_callback(
      context.callback.FileSystemFileEntry_file_success
        .get().unwrap_or_else(|| alert::error(context, "在回调 FileSystemDirectoryEntry_getFile_success 中，发现回调 FileSystemFileEntry_file_success 尚未初始化。"))
        .as_ref().unchecked_ref(),
      context.callback.FileSystemFileEntry_file_error
        .get().unwrap_or_else(|| alert::error(context, "在回调 FileSystemDirectoryEntry_getFile_success 中，发现回调 FileSystemFileEntry_file_error 尚未初始化。"))
        .as_ref().unchecked_ref(),
    );
  })) {
    alert::error(context, "重复初始化回调 FileSystemDirectoryEntry_getFile_success");
  };
}

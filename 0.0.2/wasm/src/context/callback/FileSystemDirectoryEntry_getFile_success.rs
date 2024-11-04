use std::rc::Rc;

use crate::{alert, prelude::*};

use super::super::Context;

pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.callback.FileSystemDirectoryEntry_getFile_success.set(Closure::<dyn Fn(web_sys::FileSystemFileEntry)>::new(move |file_entry: web_sys::FileSystemFileEntry| {
    alert::error(&context_clone, "not implemented");
  })) {
    alert::error(context, "重复初始化回调 FileSystemDirectoryEntry_getFile_success");
  };
}

use std::rc::Rc;

use crate::{alert, prelude::*};

use super::super::Context;

pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.callback.FileSystemDirectoryReader_readEntries_success.set(Closure::<dyn Fn(js_sys::Array)>::new(move |entries: js_sys::Array| {
    alert::error(&context_clone, "not implemented");
  })) {
    alert::error(context, "重复初始化回调 FileSystemDirectoryReader_readEntries_success");
  };
}

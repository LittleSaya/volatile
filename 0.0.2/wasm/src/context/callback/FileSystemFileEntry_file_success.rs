use std::rc::Rc;

use crate::{alert, prelude::*};

use super::super::Context;

pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.callback.FileSystemFileEntry_file_success.set(Closure::<dyn Fn(web_sys::File)>::new(move |file: web_sys::File| {
    alert::error(&context_clone, "not implemented");
  })) {
    alert::error(context, "重复初始化回调 FileSystemFileEntry_file_success");
  };
}

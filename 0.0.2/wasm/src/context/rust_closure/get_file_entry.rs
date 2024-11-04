use std::rc::Rc;

use crate::{alert, prelude::*};

use super::super::Context;

pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.rust_closure.get_file_entry.set(Box::new(move |path: String| {
    alert::error(&context_clone, "not implemented");
  })) {
    alert::error(context, "重复初始化 rust 闭包 get_file_entry");
  };
}

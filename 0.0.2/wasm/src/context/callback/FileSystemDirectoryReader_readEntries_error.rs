use std::rc::Rc;

use crate::{alert, prelude::*};

use super::super::Context;

pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.callback.FileSystemDirectoryReader_readEntries_error.set(Closure::<dyn Fn(web_sys::DomException)>::new(move |err: web_sys::DomException| {
    let context = &context_clone;
    alert::error(context, &format!("在深度遍历用户拖放的文件时遇到了一个无法读取其内容的目录，由于程序编写方式的限制，此处无法提供具体的目录名称。上游错误： {:?}", err));
  })) {
    alert::error(context, "重复初始化回调 FileSystemDirectoryReader_readEntries_error");
  };
}

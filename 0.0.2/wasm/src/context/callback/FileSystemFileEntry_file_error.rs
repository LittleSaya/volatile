use std::rc::Rc;

use crate::{alert, prelude::*};

use super::super::Context;

pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.callback.FileSystemFileEntry_file_error.set(Closure::<dyn Fn(web_sys::DomException)>::new(move |err: web_sys::DomException| {
    let context = &context_clone;
    alert::error(context, &format!("无法通过 FileSystemFileEntry 对象获取 File 对象，由于程序编写方式的限制，此处无法提供具体的文件名称。上游错误：{:?} 。", err));
  })) {
    alert::error(context, "重复初始化回调 FileSystemFileEntry_file_error");
  };
}

use std::rc::Rc;

use crate::{alert, prelude::*};

use super::super::Context;

/// Convert each item in entries array into FileSystemEntry, then call "scan" again.
pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.callback.FileSystemDirectoryReader_readEntries_success.set(Closure::<dyn Fn(js_sys::Array)>::new(move |entries: js_sys::Array| {
    let context = &context_clone;

    let len = entries.length();
    let mut v = Vec::<web_sys::FileSystemEntry>::with_capacity(len as usize);
    for i in 0..len {
      let entry = entries.get(i).unchecked_into::<web_sys::FileSystemEntry>();
      v.push(entry);
    }
    let entries = v;

    *context.scan_stage.unresolved_directories.borrow_mut() -= 1;

    context.rust_closure.scan.get().as_ref().unwrap_or_else(|| alert::error(context, "在深度遍历用户拖放的文件时，发现 rust 闭包 scan 尚未初始化。"))(entries);
  })) {
    alert::error(context, "重复初始化回调 FileSystemDirectoryReader_readEntries_success");
  };
}

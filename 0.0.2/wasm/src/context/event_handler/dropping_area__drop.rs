use std::rc::Rc;

use crate::{alert, prelude::*};

use super::super::Context;

pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.event_handler.dropping_area__drop.set(Closure::<dyn Fn(web_sys::DragEvent)>::new(move |ev: web_sys::DragEvent| {
    let context = &context_clone;

    ev.prevent_default();
    if let Some(data_transfer) = ev.data_transfer() {
      let mut entries = Vec::<web_sys::FileSystemEntry>::new();
      let items = data_transfer.items();
      for index in 0..items.length() {
        let Some(item) = items.get(index) else {
          alert::error(context, &format!("在第一次遍历用户拖放的文件时，发现索引为 {} 的 DataTransferItem 为 None 。", index));
        };

        let entry = match item.webkit_get_as_entry() {
          Ok(entry) => match entry {
            Some(entry) => entry,
            None => alert::error(context, &format!("在第一次遍历用户拖放的文件时，发现由索引为 {} 的 DataTransferItem 转换为的 FileSystemEntry 为 None 。", index)),
          },
          Err(e) => alert::error(context, &format!("在第一次遍历用户拖放的文件时，发现索引为 {} 的 DataTransferItem 无法转换为 FileSystemEntry 。上游错误： {:?} 。", index, e)),
        };

        entries.push(entry);
      }

      // initialize "scan" stage
      context.scan_stage.file_system.replace(Some(
        match entries.get(0) {
          Some(entry) => entry.filesystem(),
          None => alert::error(context, "用户拖入的文件个数为 0 。"),
        }
      ));
      context.scan_stage.file_path_list.borrow_mut().clear();
      context.scan_stage.unresolved_directories.replace(0);
      context.rust_closure.scan.get().unwrap_or_else(|| alert::error(context, "在第一次遍历用户拖放的文件时，发现 rust 闭包 scan 尚未初始化。"))(entries);
    };
  })) {
    alert::error(context, "重复初始化事件处理器 dropping_area__drop");
  };

  if let Err(e) = context.element.dropping_area.add_event_listener_with_callback(
    "drop",
    context.event_handler.dropping_area__drop.get().unwrap().as_ref().unchecked_ref()
  ) {
    alert::error(context, &format!("无法在 dropping_area 元素上注册 drop 事件处理器。上游错误：{:?}", e));
  };
}

use std::rc::Rc;

use crate::{alert, prelude::*};

use super::super::Context;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = ["window"])]
  fn create_stream_writer() -> web_sys::WritableStreamDefaultWriter;
}

pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.event_handler.compress_encrypt__click.set(Closure::<dyn Fn(web_sys::PointerEvent)>::new(move |_ev: web_sys::PointerEvent| {
    let context = &context_clone;

    // make sure "scan" stage is complete
    if context.scan_stage.file_system.borrow().is_none() {
      alert::info(context, "文件系统尚未准备完成，请先拖入文件。");
      return;
    }
    if context.scan_stage.file_path_list.borrow().len() == 0 {
      alert::info(context, "文件列表为空，请先拖入文件。");
      return;
    };
    if *context.scan_stage.unresolved_directories.borrow() != 0 {
      alert::info(context, "请等待扫描完成。");
      return;
    }

    // make sure "compress_encrypt" stage hasn't started yet
    // the sign of a started "compress_encrypt" stage is a created writer
    if context.compress_encrypt_stage.writer.borrow().is_some() {
        alert::info(context, "输出流已存在，请等待当前输出流关闭。");
        return;
    }

    context.compress_encrypt_stage.writer.replace(Some(create_stream_writer()));
    context.compress_encrypt_stage.number_compressed.replace(0);
    context.compress_encrypt_stage.file_headers.borrow_mut().clear();
    context.compress_encrypt_stage.bytes_written.replace(0);

    // the whole procedure of the "compress_encrypt" stage is composed of several closures which will invoke each other
    // "take_item" is the entry of this procedure
    context.rust_closure.take_item.get().unwrap_or_else(|| alert::error(context, "在准备进入 compress_encrypt 阶段时，发现 rust 闭包 take_item 尚未初始化。"))();
  })) {
    alert::error(context, "重复初始化事件处理器 compress_encrypt__click");
  };

  if let Err(e) = context.element.compress_encrypt.add_event_listener_with_callback(
    "click",
    context.event_handler.compress_encrypt__click.get().unwrap().as_ref().unchecked_ref()
  ) {
    alert::error(context, &format!("无法在 compress_encrypt 元素上注册 click 事件处理器。上游错误：{:?}", e));
  };
}

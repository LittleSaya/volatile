use std::rc::Rc;

use crate::{alert, prelude::*};

use super::super::Context;

pub fn init(context: &Rc<Context>) {
  // let context_clone = Rc::clone(context);
  if let Err(_) = context.event_handler.dropping_area__dragover.set(Closure::<dyn Fn(web_sys::DragEvent)>::new(move |ev: web_sys::DragEvent| {
    ev.prevent_default();
    ev.data_transfer().and_then(|data_transfer| Some(data_transfer.set_drop_effect("move")));
  })) {
    alert::error(context, "重复初始化事件处理器 dropping_area__dragover");
  };

  if let Err(e) = context.element.dropping_area.add_event_listener_with_callback(
    "dragover",
    context.event_handler.dropping_area__dragover.get().unwrap().as_ref().unchecked_ref()
  ) {
    alert::error(context, &format!("无法在 dropping_area 元素上注册 dragover 事件处理器。上游错误：{:?}", e));
  };
}

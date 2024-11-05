use std::rc::Rc;

use crate::{alert, appnote63, prelude::*, utils};

use super::super::Context;

/// Create and store the file header, then write the local file header, then move to the file content.
pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.rust_closure.get_file_entry.set(Box::new(move |path: String| {
    let context = &context_clone;

    let file_header = appnote63::FileHeader::new(
      path.trim_start_matches('/').to_owned(),
      *context.compress_encrypt_stage.bytes_written.borrow()
    );
    context.compress_encrypt_stage.file_headers.borrow_mut().push(file_header);

    let context_clone_clone = Rc::clone(&context_clone);
    wasm_bindgen_futures::spawn_local(async move {
      let context = &context_clone_clone;

      // write "local file header"
      // context_refmut.check_buffer(); TODO: figure out why
      let file_headers = context.compress_encrypt_stage.file_headers.borrow();
      let file_header = file_headers.last().unwrap_or_else(|| alert::error(context, &format!("在写入 Local File Header 时，发现 file_headers 为空，文件路径：{} 。", path)));

      let lfh_view = file_header.write_into_as_lfh(&context.compress_encrypt_stage.buffer_header.borrow());

      if let Err(e) = utils::await_promise(
        context.compress_encrypt_stage.writer
          .borrow().as_ref().unwrap_or_else(|| alert::error(context, &format!("在写入 Local File Header 时，发现 writer 尚未创建，文件路径：{} 。", path)))
          .write_with_chunk(&lfh_view)
      ).await {
        alert::error(context, &format!("Local File Header 写入失败，文件路径：{}。上游错误： {:?}", path, e));
      }

      *context.compress_encrypt_stage.bytes_written.borrow_mut() += lfh_view.byte_length() as u64;

      // continue to get the content of the file
      context.scan_stage.file_system
        .borrow().as_ref().unwrap_or_else(|| alert::error(context, &format!("在准备获取文件的 FileSystemFileEntry 时，发现文件系统尚未初始化，文件路径：{} 。", path)))
        .root().get_file_with_path_and_options_and_callback_and_callback(
          Some(&path),
          &web_sys::FileSystemFlags::default(),
          context.callback.FileSystemDirectoryEntry_getFile_success
            .get().unwrap_or_else(|| alert::error(context, &format!("在准备获取文件的 FileSystemFileEntry 时，发现回调 FileSystemDirectoryEntry_getFile_success 尚未初始化。")))
            .as_ref().unchecked_ref(),
          context.callback.FileSystemDirectoryEntry_getFile_error
            .get().unwrap_or_else(|| alert::error(context, &format!("在准备获取文件的 FileSystemFileEntry 时，发现回调 FileSystemDirectoryEntry_getFile_error 尚未初始化。")))
            .as_ref().unchecked_ref(),
        );
    });
  })) {
    alert::error(context, "重复初始化 rust 闭包 get_file_entry");
  };
}

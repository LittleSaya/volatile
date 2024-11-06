use std::rc::Rc;

use crate::{alert, constant::{TAIL_BUFFER_INITIAL_SIZE, VERSION_MADE_BY, VERSION_NEEDED_TO_EXTRACT}, prelude::*, utils};

use super::super::Context;

/// Write central directory headers, then close the writer.
pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.rust_closure.finish.set(Box::new(move || {
    let context = &context_clone;

    let start_of_central_directory = *context.compress_encrypt_stage.bytes_written.borrow();

    let mut tail_buffer = Vec::<u8>::with_capacity(TAIL_BUFFER_INITIAL_SIZE as usize);

    // central directory headers
    let file_headers = context.compress_encrypt_stage.file_headers.borrow();
    for file_header in file_headers.iter() {
      file_header.write_into_as_cdh(&mut tail_buffer);
    }

    web_sys::console::log_1(&JsValue::from_str(&format!("文件总数： {}", file_headers.len())));

    // the size of all central directory headers
    let size_of_central_directory = tail_buffer.len() as u64;

    web_sys::console::log_1(&JsValue::from_str(&format!("Central Directory 的大小： {} B", size_of_central_directory)));

    let relative_offset_of_zip64_end_of_central_directory_record = start_of_central_directory + size_of_central_directory;

    // zip64 end of central directory record
    tail_buffer.extend_from_slice(&0x06064b50_u32.to_le_bytes()); // zip64 end of central dir signature
    tail_buffer.extend_from_slice(&(2_u64 + 2_u64 + 4_u64 + 4_u64 + 8_u64 + 8_u64 + 8_u64 + 8_u64).to_le_bytes());
    tail_buffer.extend_from_slice(&VERSION_MADE_BY.to_le_bytes());
    tail_buffer.extend_from_slice(&VERSION_NEEDED_TO_EXTRACT.to_le_bytes());
    tail_buffer.extend_from_slice(&0_u32.to_le_bytes()); // splitting is not supported
    tail_buffer.extend_from_slice(&0_u32.to_le_bytes()); // splitting is not supported
    tail_buffer.extend_from_slice(&(file_headers.len() as u64).to_le_bytes());
    tail_buffer.extend_from_slice(&(file_headers.len() as u64).to_le_bytes());
    tail_buffer.extend_from_slice(&size_of_central_directory.to_le_bytes());
    tail_buffer.extend_from_slice(&start_of_central_directory.to_le_bytes());

    // zip64 end of central directory locator
    tail_buffer.extend_from_slice(&0x07064b50_u32.to_le_bytes()); // zip64 end of central dir locator signature
    tail_buffer.extend_from_slice(&0_u32.to_le_bytes()); // splitting is not supported
    tail_buffer.extend_from_slice(&relative_offset_of_zip64_end_of_central_directory_record.to_le_bytes());
    tail_buffer.extend_from_slice(&1_u32.to_le_bytes()); // splitting is not supported

    // End of central directory record
    tail_buffer.extend_from_slice(&0x06054b50_u32.to_le_bytes());
    tail_buffer.extend_from_slice(&0xFFFF_u16.to_le_bytes()); // zip64, number of this disk
    tail_buffer.extend_from_slice(&0xFFFF_u16.to_le_bytes()); // zip64, number of the disk with the start of the central directory
    tail_buffer.extend_from_slice(&0xFFFF_u16.to_le_bytes()); // zip64, total number of entries in the central directory on this disk
    tail_buffer.extend_from_slice(&0xFFFF_u16.to_le_bytes()); // zip64, total number of entries in the central directory
    tail_buffer.extend_from_slice(&0xFFFFFFFF_u32.to_le_bytes()); // zip64, size of the central directory
    tail_buffer.extend_from_slice(&0xFFFFFFFF_u32.to_le_bytes()); // zip64, offset of start of central directory with respect to the starting disk number
    tail_buffer.extend_from_slice(&0_u16.to_le_bytes()); // zip file comment length
    // central_directory_buffer.extend_from_slice(&[0_u8; 0]); // zip file comment (no comment)

    web_sys::console::log_1(&JsValue::from_str(&format!("尾部数据的大小： {} B", tail_buffer.len())));

    let context_clone_clone = Rc::clone(context);
    wasm_bindgen_futures::spawn_local(async move {
      let context = &context_clone_clone;

      let tail_array_buffer = js_sys::ArrayBuffer::new(tail_buffer.len() as u32);
      let tail_view = js_sys::Uint8Array::new(&tail_array_buffer);
      tail_view.copy_from(&tail_buffer);

      if let Err(e) = utils::await_promise(
        context.compress_encrypt_stage.writer
          .borrow().as_ref().unwrap_or_else(|| alert::error(context, "在写入尾部数据时，发现 writer 尚未创建。"))
          .write_with_chunk(&tail_view)
      ).await {
        alert::error(context, &format!("尾部数据写入失败。上游错误： {:?}", e));
      }

      if let Err(e) = utils::await_promise(
        context.compress_encrypt_stage.writer
          .borrow().as_ref().unwrap_or_else(|| alert::error(context, "关闭输出流时，发现 writer 尚未创建。"))
          .close()
      ).await {
        alert::error(context, &format!("未能正确关闭输出流。上游错误：{:?}", e));
      }

      context.compress_encrypt_stage.writer.replace(None);

      web_sys::console::log_1(&JsValue::from_str("尾部数据写入完成，输出流已关闭"));
    });
  })) {
    alert::error(context, "重复初始化 rust 闭包 finish");
  };
}

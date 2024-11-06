use std::{io::Write, rc::Rc};

use crc32fast::Hasher;
use flate2::{write::DeflateEncoder, Compression};

use crate::{alert, constant::BUFFER_DATA_SIZE, prelude::*, utils::{self, ReadResult}};

use super::super::Context;

/// Create BYOB reader, read data, calculate crc32, compress, write compressed data, then write data descriptor.
pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.callback.FileSystemFileEntry_file_success.set(Closure::<dyn Fn(web_sys::File)>::new(move |file: web_sys::File| {
    let context = &context_clone;

    let file_name = file.name();

    let Ok(blob) = file.dyn_into::<web_sys::Blob>()
    else {
      alert::error(context, &format!("无法将 File 对象转换为 Blob 对象，文件名：{} 。", file_name));
    };

    let stream = blob.stream();
    let reader_option = web_sys::ReadableStreamGetReaderOptions::new();
    reader_option.set_mode(web_sys::ReadableStreamReaderMode::Byob);
    let reader = stream.get_reader_with_options(&reader_option);
    let Ok(reader) = reader.dyn_into::<web_sys::ReadableStreamByobReader>()
    else {
      alert::error(context, &format!("无法将 ReadableStream::get_reader_with_options() 的返回值转换为 ReadableStreamByobReader 对象，文件名： {} 。", file_name));
    };

    let context_clone_clone = Rc::clone(&context_clone);
    wasm_bindgen_futures::spawn_local(async move {
      let context = &context_clone_clone;

      // ----------
      // file data
      // ----------

      let mut hasher = Hasher::new();
      let mut deflate_encoder = DeflateEncoder::new(Vec::with_capacity(1024 * 1024), Compression::new(5)); // 1MiB inner buffer

      let mut read_buffer = js_sys::ArrayBuffer::new(BUFFER_DATA_SIZE);
      let mut transform_buffer = vec![0_u8; BUFFER_DATA_SIZE as usize];

      let mut total_in = 0_u64;
      let mut total_out = 0_u64;

      let time_before_compress = context.performance.now();

      loop {
        // js reader -> js buffer
        let ReadResult { new_buffer, view, done } = utils::byob_read(context, &read_buffer, &reader).await;

        let bytes_read = view.byte_length();

        total_in += bytes_read as u64;

        // web_sys::console::log_1(&JsValue::from_str(&format!(
        //   "读取数据 {:>8.2} KiB ，最后一次： {}",
        //   bytes_read as f64 / 1024_f64,
        //   done,
        // )));

        read_buffer = new_buffer; // replace old detached ArrayBuffer immediately

        // if indeed read some bytes
        if bytes_read > 0 {
          // js buffer -> wasm buffer slice
          let wasm_slice = &mut context.compress_encrypt_stage.buffer_data.borrow_mut()[0..bytes_read as usize];
          view.copy_to(wasm_slice);

          // wasm buffer slice -> crc hasher
          hasher.update(wasm_slice);

          // wasm buffer slice -> deflate encoder -> inner buffer
          deflate_encoder.get_mut().clear();

          // let time_before_compress = context.performance.now();

          if let Err(e) = deflate_encoder.write_all(wasm_slice) {
            alert::error(context, &format!("压缩失败，文件名： {} 。上游错误： {:?}", file_name, e));
          }

          // let compress_cost_time = context.performance.now() - time_before_compress;

          // if indeed get some compressed bytes
          let bytes_output = deflate_encoder.get_ref().len();

          total_out += bytes_output as u64;

          // web_sys::console::log_1(&JsValue::from_str(&format!(
          //   "压缩数据 {:>8.2} KiB ，耗时 {:>8.2} S ，速度 {:>8.2} KiB/S",
          //   bytes_output as f64 / 1024_f64,
          //   compress_cost_time / 1000_f64,
          //   (bytes_output as f64 / 1024_f64) / (compress_cost_time / 1000_f64),
          // )));

          if bytes_output > 0 {
            // inner buffer -> js buffer
            let buffer_output = js_sys::ArrayBuffer::new(bytes_output as u32);
            let view_output = js_sys::Uint8Array::new_with_byte_offset_and_length(
              &buffer_output,
              0,
              bytes_output as u32,
            );
            view_output.copy_from(deflate_encoder.get_ref());

            // js buffer -> js writer
            if let Err(e) = utils::await_promise(
              context.compress_encrypt_stage.writer
                .borrow().as_ref().unwrap_or_else(|| alert::error(context, "在写入压缩数据时，发现 writer 尚未创建。"))
                .write_with_chunk(&view_output)
            ).await {
              alert::error(context, &format!("压缩数据写入失败，文件名： {} 。上游错误： {:?}", file_name, e));
            }
          }
        }

        if done {
          // deflate encoder -> inner buffer
          deflate_encoder.get_mut().clear();
          let inner_buffer = match deflate_encoder.finish() {
            Ok(w) => w,
            Err(e) => alert::error(context, &format!("（最后一次）压缩失败，文件名： {} 。上游错误： {:?}", file_name, e)),
          };

          let bytes_output = inner_buffer.len();

          total_out += bytes_output as u64;

          // if indeed we get some final extra compressed bytes to write
          if bytes_output > 0 {
            // inner buffer -> js buffer
            let buffer_output = js_sys::ArrayBuffer::new(bytes_output as u32);
            let view_output = js_sys::Uint8Array::new_with_byte_offset_and_length(
              &buffer_output,
              0,
              bytes_output as u32,
            );
            view_output.copy_from(&inner_buffer);

            // js buffer -> js writer
            if let Err(e) = utils::await_promise(
              context.compress_encrypt_stage.writer
                .borrow().as_ref().unwrap_or_else(|| alert::error(context, "在最后一次写入压缩数据时，发现 writer 尚未创建。"))
                .write_with_chunk(&view_output)
            ).await {
              alert::error(context, &format!("压缩数据（最后一批）写入失败，文件名： {} 。上游错误： {:?}", file_name, e));
            }
          }

          break;
        }
      }

      // web_sys::console::log_1(&JsValue::from_str(&format!("压缩数据已写入，大小 {} B", total_out)));

      let compress_time_cost = context.performance.now() - time_before_compress;

      web_sys::console::log_1(&JsValue::from_str(&format!(
        "压缩完成，压缩后大小 {:>8.2} KiB ，压缩前大小 {:>8.2} KiB ，耗时 {:>8.2} S ，速度 {:>8.2} KiB/S",
        total_out as f64 / 1024_f64,
        total_in as f64 / 1024_f64,
        compress_time_cost / 1000_f64,
        (total_in as f64 / 1024_f64) / (compress_time_cost / 1000_f64)
      )));

      // ----------
      // data descriptor
      // ----------

      let crc32 = hasher.finalize();

      const DATA_DESCRIPTOR_SIZE: u32 = 4 + 4 + 8 + 8;

      let mut data_descriptor_buffer = Vec::<u8>::with_capacity(DATA_DESCRIPTOR_SIZE as usize);
      data_descriptor_buffer.extend_from_slice(&0x08074b50_u32.to_le_bytes());
      data_descriptor_buffer.extend_from_slice(&crc32.to_le_bytes());
      data_descriptor_buffer.extend_from_slice(&total_out.to_le_bytes());
      data_descriptor_buffer.extend_from_slice(&total_in.to_le_bytes());

      // wasm buffer -> js buffer
      let data_descriptor_array_buffer = read_buffer; // reuse read buffer
      let data_descriptor_view = js_sys::Uint8Array::new_with_byte_offset_and_length(
        &data_descriptor_array_buffer,
        0,
        DATA_DESCRIPTOR_SIZE
      );
      data_descriptor_view.copy_from(&data_descriptor_buffer);

      // js buffer -> js writer
      if let Err(e) = utils::await_promise(
        context.compress_encrypt_stage.writer
          .borrow().as_ref().unwrap_or_else(|| alert::error(context, "在写入 Data Descriptor 时，发现 writer 尚未创建。"))
          .write_with_chunk(&data_descriptor_view)
      ).await {
        alert::error(context, &format!("Data Descriptor 写入失败，文件名： {} 。上游错误： {:?}", file_name, e));
      }

      web_sys::console::log_1(&JsValue::from_str(&format!("Data Descriptor 已写入，大小 {} B", DATA_DESCRIPTOR_SIZE)));

      let old_bytes_written = *context.compress_encrypt_stage.bytes_written.borrow();
      let new_bytes_written = old_bytes_written + total_out + DATA_DESCRIPTOR_SIZE as u64;
      *context.compress_encrypt_stage.bytes_written.borrow_mut() = new_bytes_written;

      web_sys::console::log_1(&JsValue::from_str(&format!("当前写入量 {} B", new_bytes_written)));

      // set crc_32 of the last file header (current file header) to calculated value
      let mut file_headers = context.compress_encrypt_stage.file_headers.borrow_mut();
      let current_file_header = file_headers.last_mut().unwrap_or_else(|| alert::error(context, &format!("在写回 crc32 时，发现 file_headers 为空，文件名： {} 。", file_name)));
      current_file_header.set_crc_32(crc32);
      current_file_header.set_compressed_size_u64(total_out);
      current_file_header.set_uncompressed_size_u64(total_in);
      drop(file_headers);

      context.rust_closure.take_item.get().unwrap_or_else(|| alert::error(context, "在回调 FileSystemFileEntry_file_success 的末尾，发现 rust 闭包 take_item 尚未初始化。"))();
    });
  })) {
    alert::error(context, "重复初始化回调 FileSystemFileEntry_file_success");
  };
}

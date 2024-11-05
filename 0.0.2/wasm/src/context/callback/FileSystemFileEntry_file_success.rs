use std::{io::Write, rc::Rc};

use crc32fast::Hasher;
use flate2::{write::DeflateEncoder, Compression};

use crate::{alert, prelude::*, utils};

use super::super::Context;

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
      alert::error(context, &format!("无法将 ReadableStream::get_reader_with_options() 的返回值转换为 ReadableStreamByobReader 对象，文件名：{} 。", file_name));
    };

    let context_clone_clone = Rc::clone(&context_clone);
    wasm_bindgen_futures::spawn_local(async move {
      let context = &context_clone_clone;

      let mut hasher = Hasher::new();
      let mut deflate_encoder = DeflateEncoder::new(Vec::with_capacity(1024 * 1024), Compression::new(5)); // 1MiB inner buffer
      let mut crc32 = 0_u32;

      let buffer_0 = js_sys::ArrayBuffer::new(1024 * 1024);

      loop {
        // js reader -> js buffer
        // context.check_buffer(); TODO: figure out why
        // let buffer_data = js_sys::ArrayBuffer::new(1024 * 1024);
        // let view_data = js_sys::Uint8Array::new(&context.compress_encrypt_stage.buffer_data.borrow());
        // let view_data = js_sys::Uint8Array::new(&buffer_data);
        web_sys::console::log_1(&JsValue::from_bool(buffer_0.byte_length() == 0));
        let view_data = js_sys::Uint8Array::new(&buffer_0);
        let view_data_real = match utils::await_promise(reader.read_with_array_buffer_view(&view_data)).await {
          Ok(res) => {
            web_sys::console::log_1(&JsValue::from_bool(buffer_0.byte_length() == 0));
            let value = match js_sys::Reflect::get(&res, &"value".into()) {
              Ok(v) => v,
              Err(e) => alert::error(context, &format!("无法从 ReadableStreamByobReader::read_with_array_buffer_view() 的返回值中提取 value 属性，文件名：{} 。上游错误： {:?}", file_name, e)),
            };
            let done = match js_sys::Reflect::get(&res, &"done".into()) {
              Ok(v) => v,
              Err(e) => alert::error(context, &format!("无法从 ReadableStreamByobReader::read_with_array_buffer_view() 的返回值中提取 done 属性，文件名：{} 。上游错误： {:?}", file_name, e)),
            };

            if value.eq(&JsValue::UNDEFINED) {
              alert::error(context, &format!("输入流已经被取消，文件名： {} 。", file_name));
            }

            let done = done.eq(&JsValue::TRUE);

            let value = match value.dyn_into::<js_sys::Uint8Array>() {
              Ok(v) => v,
              Err(_) => alert::error(context, &format!("无法将 ReadableStreamByobReader::read_with_array_buffer_view() 的返回值中的 value 属性转换为 Uint8Array 对象，文件名：{}。", file_name)),
            };

            (value, done)
          },
          Err(e) => alert::error(context, &format!("文件流读取失败，文件名： {} 。上游错误： {:?}", file_name, e)),
        };

        // if indeed read some bytes
        if view_data_real.0.byte_length() > 0 {
          // js buffer -> wasm buffer
          // TODO: this is probably the first location where a Uint8Array is constructed over the wasm memory, which might cause "detached arraybuffer" error
          let wasm_slice = &mut context.compress_encrypt_stage.buffer_data_wasm.borrow_mut()[0..view_data_real.0.byte_length() as usize];
          view_data_real.0.copy_to(wasm_slice);

          // wasm buffer -> crc hasher
          hasher.update(wasm_slice);

          // wasm buffer -> deflate encoder -> inner buffer
          deflate_encoder.get_mut().clear();
          if let Err(e) = deflate_encoder.write_all(wasm_slice) {
            alert::error(context, &format!("压缩失败，文件名： {} 。上游错误： {:?}", file_name, e));
          }

          // if indeed get some compressed bytes
          if deflate_encoder.get_ref().len() > 0 {
            // inner buffer -> js buffer (reuse the input buffer)
            // context.check_buffer(); TODO: figure out why
            // let buffer_output = js_sys::ArrayBuffer::new(1024 * 1024);
            // let view_output = js_sys::Uint8Array::new_with_byte_offset_and_length(
            //   &context.compress_encrypt_stage.buffer_data.borrow(),
            //   0,
            //   deflate_encoder.get_ref().len() as u32
            // );
            // let view_output = js_sys::Uint8Array::new_with_byte_offset_and_length(&buffer_output, 0, deflate_encoder.get_ref().len() as u32);
            let view_output = js_sys::Uint8Array::new_with_byte_offset_and_length(&buffer_0, 0, deflate_encoder.get_ref().len() as u32);
            // TODO: this is probably the second position where a Uint8Array is constructed over the wasm memory
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

        // if done
        if view_data_real.1 {
          crc32 = hasher.finalize();

          // deflate encoder -> inner buffer
          deflate_encoder.get_mut().clear();
          if let Err(e) = deflate_encoder.flush() {
            alert::error(context, &format!("（最后一次）压缩失败，文件名： {} 。上游错误： {:?}", file_name, e));
          }

          // if indeed we get some extra compressed bytes to write
          if deflate_encoder.get_ref().len() > 0 {
            // inner buffer -> js buffer (reuse the input buffer again)
            // context.check_buffer(); TODO: figure out why
            // let buffer_output = js_sys::ArrayBuffer::new(1024 * 1024);
            // let view_output = js_sys::Uint8Array::new_with_byte_offset_and_length(
            //   &context.compress_encrypt_stage.buffer_data.borrow(),
            //   0,
            //   deflate_encoder.get_ref().len() as u32
            // );
            // let view_output = js_sys::Uint8Array::new_with_byte_offset_and_length(&buffer_output, 0, deflate_encoder.get_ref().len() as u32);
            let view_output = js_sys::Uint8Array::new_with_byte_offset_and_length(&buffer_0, 0, deflate_encoder.get_ref().len() as u32);
            // TODO: this is probably the third (and the last) position where a Uint8Array is constructed over the wasm memory
            view_output.copy_from(deflate_encoder.get_ref());

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

      *context.compress_encrypt_stage.bytes_written.borrow_mut() += deflate_encoder.total_out();

      web_sys::console::log_1(&JsValue::from_f64(*context.compress_encrypt_stage.bytes_written.borrow() as f64));

      // TODO: write data descriptor

      context.rust_closure.take_item.get().unwrap_or_else(|| alert::error(context, "在回调 FileSystemFileEntry_file_success 的末尾，发现 rust 闭包 take_item 尚未初始化。"))();
    });
  })) {
    alert::error(context, "重复初始化回调 FileSystemFileEntry_file_success");
  };
}

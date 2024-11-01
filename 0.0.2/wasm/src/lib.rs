use std::{cell::RefCell, rc::Rc};

use web_sys::{wasm_bindgen, FileSystemEntry, HtmlDivElement};
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::js_sys;

mod utils;
mod appnote63;
mod writer_chain;

struct FileItem {
    path: String,
    file: Option<web_sys::File>,
}

#[wasm_bindgen]
pub async fn init(
    dropping_area_element_id: String,
    status_element_id: String,
    compress_encrypt_element_id: String,
    decrypt_element_id: String,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let file_item_list = Rc::new(RefCell::new(Vec::<FileItem>::new()));

    let Ok(dropping_area_element) =
        validate_element::<web_sys::HtmlDivElement>(&document, &dropping_area_element_id)
        else {
            return Err("dropping area 元素异常".into());
        };
    let dropping_area_element = Rc::new(dropping_area_element);

    let Ok(status_element) =
        validate_element::<web_sys::HtmlDivElement>(&document, &status_element_id)
        else {
            return Err("status 元素异常".into());
        };
    let status_element = Rc::new(status_element);

    let Ok(compress_encrypt_element) =
        validate_element::<web_sys::HtmlButtonElement>(&document, &compress_encrypt_element_id)
        else {
            return Err("compress encrypt 元素异常".into());
        };
    
    let Ok(decrypt_element) =
        validate_element::<web_sys::HtmlButtonElement>(&document, &decrypt_element_id)
        else {
            return Err("decrypt 元素异常".into());
        };

    if let Err(e) = init_drag_and_drop(Rc::clone(&dropping_area_element), Rc::clone(&file_item_list), Rc::clone(&status_element)) {
        return Err(e);
    }

    Ok("WASM 业务逻辑初始化成功".into())
}

/// Check the existence and type of a specific element.
fn validate_element<T: wasm_bindgen::JsCast>(document: &web_sys::Document, element_id: &str) -> Result<T, ()> {
    use wasm_bindgen::JsCast;

    let Some(element) = document.get_element_by_id(&element_id) else { return Err(()); };
    let Ok(typed_element) = element.dyn_into::<T>() else { return Err(()); };
    Ok(typed_element)
}

fn init_drag_and_drop(
    dropping_area: Rc<web_sys::HtmlDivElement>,
    file_item_list: Rc<RefCell<Vec<FileItem>>>,
    status_element: Rc<web_sys::HtmlDivElement>
) -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    let dragover_handler = Closure::<dyn Fn(web_sys::DragEvent)>::new(|ev: web_sys::DragEvent| {
        ev.prevent_default();
        ev.data_transfer().and_then(|data_transfer| Some(data_transfer.set_drop_effect("move")));
    });
    dropping_area.add_event_listener_with_callback("dragover", dragover_handler.as_ref().unchecked_ref())?;
    dragover_handler.forget();
    
    let drop_handler = Closure::<dyn Fn(web_sys::DragEvent)>::new(move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        ev.data_transfer().and_then(|data_transfer| {
            let mut entries = Vec::<FileSystemEntry>::new();
            let items = data_transfer.items();
            for index in 0..items.length() {
                let Some(item) = items.get(index)
                else {
                    web_sys::window().unwrap()
                        .alert_with_message(&format!(
                            "在遍历用户拖放的文件时，发现索引为 {} 的 DataTransferItem 为 None ，程序无法继续执行，如果希望重试的话请刷新页面",
                            index
                        ))
                        .unwrap();
                    panic!();
                };

                let Ok(Some(entry)) = item.webkit_get_as_entry()
                else {
                    web_sys::window().unwrap()
                        .alert_with_message(&format!(
                            "在遍历用户拖放的文件时，发现索引为 {} 的 DataTransferItem 无法转换为 FileSystemEntry ，程序无法继续执行，如果希望重试的话请刷新页面",
                            index
                        ))
                        .unwrap();
                    panic!();
                };

                entries.push(entry);
            }

            fn dfs(entries: Vec<FileSystemEntry>, file_item_list: Rc<RefCell<Vec<FileItem>>>, status_element: Rc<HtmlDivElement>) {
                for entry in entries {
                    let full_path = entry.full_path();

                    if entry.is_file() {
                        let Ok(file_entry) = entry.dyn_into::<web_sys::FileSystemFileEntry>()
                        else {
                            web_sys::window().unwrap()
                                .alert_with_message(&format!(
                                    // "在遍历用户拖放的文件时，无法将路径为 {} 的 FileSystemEntry 转换为 FileSystemFileEntry ，程序无法继续执行，如果希望重试的话请刷新页面",
                                    "when traversing files dropped by user, can not convert FileSystemEntry at {} to FileSystemFileEntry",
                                    full_path
                                ))
                                .unwrap();
                            panic!();
                        };

                        let file_item_list_clone = Rc::clone(&file_item_list);
                        let full_path_clone = full_path.clone();
                        let status_element_clone = Rc::clone(&status_element);

                        let success_callback = Closure::<dyn Fn(web_sys::File)>::new(move |file: web_sys::File| {
                            let mut file_item_list_bor = file_item_list_clone.borrow_mut();
                            file_item_list_bor.push(FileItem {
                                path: full_path_clone.clone(),
                                file: Some(file),
                            });

                            status_element_clone.set_inner_text(&(status_element_clone.inner_text().parse::<u32>().unwrap() + 1).to_string());
                        });

                        let error_callback = Closure::<dyn Fn(web_sys::DomException)>::new(move |err: web_sys::DomException| {
                            web_sys::window().unwrap()
                                .alert_with_message(&format!(
                                    "在遍历用户拖放的文件时，无法从路径为 {} 的 FileSystemFileEntry 中获取 File ，程序无法继续执行，如果希望重试的话请刷新页面，错误详情： {:?}",
                                    full_path,
                                    err,
                                ))
                                .unwrap();
                            panic!();
                        });

                        file_entry.file_with_callback_and_callback(
                            success_callback.as_ref().unchecked_ref(),
                            error_callback.as_ref().unchecked_ref()
                        );

                        success_callback.forget();
                        error_callback.forget();
                    }
                }
            }

            let mut file_item_list_bor = (file_item_list).borrow_mut();
            file_item_list_bor.clear();
            drop(file_item_list_bor);

            dfs(entries, Rc::clone(&file_item_list), Rc::clone(&status_element));

            Some(())
        });
    });
    dropping_area.add_event_listener_with_callback("drop", drop_handler.as_ref().unchecked_ref())?;
    drop_handler.forget();

    Ok(())
}

// #[wasm_bindgen]
// pub async fn push_file(
//     path: String,
//     reader: &web_sys::ReadableStreamByobReader,
//     writer: &web_sys::WritableStreamDefaultWriter
// ) -> Result<JsValue, JsValue> {
//     let file_header = appnote63::FileHeader::new(path, context.bytes_written);

//     // write "local file header"
//     let record_view = unsafe { Uint8Array::view(&context.record_buffer) };
//     let record_view_size = file_header.write_into_as_lfh(&record_view);
//     let lfh_view = record_view.subarray(0, record_view_size);
//     let lfh_promise = writer.write_with_chunk(&lfh_view);
//     if let Err(e) = wasm_bindgen_futures::JsFuture::from(lfh_promise).await {
//         return Err(e);
//     }

//     // write "file data"
//     // prepare writer chain
//     let transform_writer = TransformWriter::new();
//     let compress_writer = CompressWriter::new(transform_writer);
//     let mut crc_writer = CrcWriter::new(compress_writer);

//     loop {
//         // TODO: one extra copy here, might be unnecessary
//         // reader -> js buffer -> rust buffer
//         let is_done;
//         let read_view = unsafe { Uint8Array::view(&context.read_buffer) };
//         let read_promise = reader.read_with_array_buffer_view(&read_view);
//         let read_result = match wasm_bindgen_futures::JsFuture::from(read_promise).await {
//             Ok(read_result) => {
//                 let value = match Reflect::get(&read_result, &"value".into()) {
//                     Ok(v) => v,
//                     Err(_e) => return Err(js_sys::JsString::from("field \"value\" is expected in the return object of ReadableStreamByobReader::read_with_array_buffer_view()").into()),
//                 };
//                 let done = match Reflect::get(&read_result, &"done".into()) {
//                     Ok(v) => v,
//                     Err(_e) => return Err(js_sys::JsString::from("field \"done\" is expected in the return object of ReadableStreamByobReader::read_with_array_buffer_view()").into()),
//                 };

//                 if value.eq(&JsValue::undefined()) {
//                     return Err(js_sys::JsString::from("reader's stream is cancelled").into());
//                 }

//                 is_done = done.eq(&JsValue::TRUE);

//                 match value.dyn_into::<Uint8Array>() {
//                     Ok(v) => v,
//                     Err(_e) => return Err(js_sys::JsString::from("Fail to cast the return value of ReadableStreamByobReader::read_with_array_buffer_view to Uint8Array").into()),
//                 }
//             },
//             Err(e) => return Err(e),
//         };
//         let rust_slice = &mut context.rust_buffer[0..view_plain.length() as usize];
//         view_plain.copy_to(rust_slice);

//         // rust buffer -> crc writer -> compress writer -> transform writer
//         if let Err(e) = crc_writer.write_all(rust_slice) {
//             return Err(js_sys::JsString::from(format!("{e}")).into());
//         }

//         // last buffer -> js buffer -> writer
//         let last_buffer = crc_writer.last_buffer();
//         let view_processed = Uint8Array::new_with_byte_offset_and_length(&context.js_buffer, 0, last_buffer.len() as u32);
//         view_processed.copy_from(last_buffer);
//         let promise_write = writer.write_with_chunk(&view_processed);
//         if let Err(e) = wasm_bindgen_futures::JsFuture::from(promise_write).await {
//             return Err(e);
//         }

//         // clear buffers in the chain
//         crc_writer.clear_buffer();

//         if is_done {
//             // flush, and do the copying job on last time
//             if let Err(e) = crc_writer.flush() {
//                 return Err(js_sys::JsString::from(format!("Fail to flush crc_writer: {e}")).into());
//             }

//             // last buffer -> js buffer -> writer
//             let last_buffer = crc_writer.last_buffer();
//             if last_buffer.len() > 0 {
//                 let view_processed = Uint8Array::new_with_byte_offset_and_length(&context.js_buffer, 0, last_buffer.len() as u32);
//                 view_processed.copy_from(last_buffer);
//                 let promise_write = writer.write_with_chunk(&view_processed);
//                 if let Err(e) = wasm_bindgen_futures::JsFuture::from(promise_write).await {
//                     return Err(e);
//                 }
//             }

//             break;
//         }
//     }

//     // write "data descriptor"

//     // modify file header

//     Ok(JsValue::undefined())
// }

// #[wasm_bindgen]
// pub async fn push_directory(
//     path: String,
//     writer: &web_sys::WritableStreamDefaultWriter
// ) -> Result<JsValue, JsValue> {
//     todo!()
// }

// #[wasm_bindgen]
// pub async fn finish(
//     writer: &web_sys::WritableStreamDefaultWriter
// ) -> Result<JsValue, JsValue> {
//     Ok(JsValue::undefined())
// }

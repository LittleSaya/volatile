//! # Execution map of "Compress & Encrypt" stage
//!
//! 1. The click handler on "compress_encrypt" element.
//! 2. The rust closure: "take_item".
//! 3. If next item is a file, go to 4.
//! 4. The rust closure: "get_file_entry".
//! 5. Inner async block.
//! 6. The callback: "directory_get_file_entry_success"
//! 7. The callback: "file_entry_success"
//! 8. Inner async block, go to 2

use std::{cell::RefCell, io::Write, mem, rc::Rc};

use web_sys::{self, js_sys, wasm_bindgen, DomException, FileSystemFlags};
use wasm_bindgen::prelude::*;
use writer_chain::{CompressWriter, CrcWriter, GetLastBuffer, TransformWriter};

mod utils;
mod appnote63;
mod writer_chain;
mod context;

const BUFFER_HEADER_SIZE: u32 = 64 * 1024; // 64 KiB
const BUFFER_DATA_SIZE: u32 = 1024 * 1024; // 1 MiB

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window"])]
    fn create_stream_writer() -> web_sys::WritableStreamDefaultWriter;
}

struct FileItem {
    path: String,
    is_file: bool,
}

struct Callback {
    file_entry_success: Option<Closure<dyn Fn(web_sys::File)>>,
    file_entry_error: Option<Closure<dyn Fn(web_sys::DomException)>>,
    directory_get_file_entry_success: Option<Closure<dyn Fn(web_sys::FileSystemFileEntry)>>,
    directory_get_file_entry_error: Option<Closure<dyn Fn(web_sys::DomException)>>,
    directory_reader_success: Option<Closure<dyn Fn(js_sys::Array)>>,
    directory_reader_error: Option<Closure<dyn Fn(web_sys::DomException)>>,
}

struct Context {
    window: web_sys::Window,
    dropping_area_element: web_sys::HtmlDivElement,
    status_element: web_sys::HtmlDivElement,
    compress_encrypt_element: web_sys::HtmlButtonElement,
    decrypt_element: web_sys::HtmlButtonElement,

    // below will be used in scan procedure
    file_system: Option<web_sys::FileSystem>,
    file_item_list: Vec<FileItem>,
    unresolved_directories: u64,

    // below will be used in compress & encrypt procedure
    writer: Option<web_sys::WritableStreamDefaultWriter>,
    number_compressed: u64,
    file_headers: Vec<appnote63::FileHeader>,
    bytes_written: u64,
    buffer_header: js_sys::ArrayBuffer,
    buffer_data: js_sys::ArrayBuffer,
    buffer_data_wasm: Vec<u8>,

    // below will be used when compressing & encrypting a single file
    current_file_header: Option<usize>,

    callback: Callback,
}

impl Context {
    pub fn check_buffer(&mut self) {
        if self.buffer_header.byte_length() == 0 {
            web_sys::console::log_1(&"header buffer detached, recreating".into());
            self.buffer_header = js_sys::ArrayBuffer::new(BUFFER_HEADER_SIZE);
        }
        if self.buffer_data.byte_length() == 0 {
            web_sys::console::log_1(&"data buffer detached, recreating".into());
            self.buffer_data = js_sys::ArrayBuffer::new(BUFFER_DATA_SIZE);
        }
    }
}

struct RustClosure {
    scan: Option<Box<dyn Fn(Vec<web_sys::FileSystemEntry>)>>,
    take_item: Option<Rc<dyn Fn()>>,
    get_file_entry: Option<Box<dyn Fn(String)>>,
    process_directory: Option<Box<dyn Fn(String)>>,
}

#[wasm_bindgen]
pub async fn main(
    dropping_area_element_id: String,
    status_element_id: String,
    compress_encrypt_element_id: String,
    decrypt_element_id: String,
) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let Ok(dropping_area_element) =
        validate_element::<web_sys::HtmlDivElement>(&document, &dropping_area_element_id)
        else {
            return Err("dropping area 元素异常".into());
        };

    let Ok(status_element) =
        validate_element::<web_sys::HtmlDivElement>(&document, &status_element_id)
        else {
            return Err("status 元素异常".into());
        };

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

    let file_item_list = Vec::<FileItem>::new();

    let context = Rc::new(RefCell::new(Context {
        window,
        dropping_area_element,
        status_element,
        compress_encrypt_element,
        decrypt_element,

        file_system: None,
        file_item_list,
        unresolved_directories: 0,

        writer: None,
        number_compressed: 0,
        file_headers: Vec::new(),
        bytes_written: 0,
        buffer_header: js_sys::ArrayBuffer::new(BUFFER_HEADER_SIZE),
        buffer_data: js_sys::ArrayBuffer::new(BUFFER_DATA_SIZE),
        buffer_data_wasm: vec![0; BUFFER_DATA_SIZE as usize],

        current_file_header: None,

        callback: Callback {
            file_entry_success: None,
            file_entry_error: None,
            directory_get_file_entry_success: None,
            directory_get_file_entry_error: None,
            directory_reader_success: None,
            directory_reader_error: None,
        },
    }));
    let mut context_refmut = context.borrow_mut();

    let rust_closure = Rc::new(RefCell::new(RustClosure {
        scan: None,
        take_item: None,
        get_file_entry: None,
        process_directory: None,
    }));
    let mut rust_closure_refmut = rust_closure.borrow_mut();

    // callback - file_entry_success

    let rust_closure_clone = Rc::clone(&rust_closure);
    let context_clone = Rc::clone(&context);
    context_refmut.callback.file_entry_success = Some(Closure::<dyn Fn(web_sys::File)>::new(move |file: web_sys::File| {
        let context_ref = context_clone.borrow();

        let file_name = file.name().clone();

        let Ok(blob) = file.dyn_into::<web_sys::Blob>()
        else {
            context_ref.window.alert_with_message(&format!("无法将 File 对象转换为 Blob 对象，文件名：{}。程序无法继续运行，若希望重试请刷新页面。", file_name)).unwrap();
            panic!();
        };

        let stream = blob.stream();
        let reader_option = web_sys::ReadableStreamGetReaderOptions::new();
        reader_option.set_mode(web_sys::ReadableStreamReaderMode::Byob);
        let reader = stream.get_reader_with_options(&reader_option);
        let Ok(reader) = reader.dyn_into::<web_sys::ReadableStreamByobReader>()
        else {
            context_ref.window.alert_with_message(&format!("无法将 ReadableStream::get_reader_with_options() 的返回值转换为 ReadableStreamByobReader 对象，文件名：{}。程序无法继续运行，若希望重试请刷新页面。", file_name)).unwrap();
            panic!();
        };

        let rust_closure_clone_clone = Rc::clone(&rust_closure_clone);
        let context_clone_clone = Rc::clone(&context_clone);
        wasm_bindgen_futures::spawn_local(async move {
            let mut context_refmut = context_clone_clone.borrow_mut();

            // prepare writer chain
            let transform_writer = TransformWriter::new();
            let compress_writer = CompressWriter::new(transform_writer);
            let mut crc_writer = CrcWriter::new(compress_writer);

            loop {
                // reader -> js buffer -> wasm buffer
                context_refmut.check_buffer();
                let prepared_view = js_sys::Uint8Array::new(&context_refmut.buffer_data);
                let read_view = match await_promise(reader.read_with_array_buffer_view(&prepared_view)).await {
                    Ok(res) => {
                        let value = match js_sys::Reflect::get(&res, &"value".into()) {
                            Ok(v) => v,
                            Err(e) => {
                                context_refmut.window.alert_with_message(
                                    &format!("无法从 ReadableStreamByobReader::read_with_array_buffer_view() 的返回值中提取 value 属性，文件名：{}。程序无法继续运行，若希望重试请刷新页面。上游错误：{:?}", file_name, e)
                                ).unwrap();
                                panic!();
                            },
                        };
                        let done = match js_sys::Reflect::get(&res, &"done".into()) {
                            Ok(v) => v,
                            Err(e) => {
                                context_refmut.window.alert_with_message(
                                    &format!("无法从 ReadableStreamByobReader::read_with_array_buffer_view() 的返回值中提取 done 属性，文件名：{}。程序无法继续运行，若希望重试请刷新页面。上游错误：{:?}", file_name, e)
                                ).unwrap();
                                panic!();
                            },
                        };

                        if value.eq(&JsValue::undefined()) {
                            context_refmut.window.alert_with_message(&format!("输入流已经被取消，文件名：{}。程序无法继续运行，若希望重试请刷新页面。", file_name)).unwrap();
                            panic!();
                        }

                        let done = done.eq(&JsValue::TRUE);

                        let value = match value.dyn_into::<js_sys::Uint8Array>() {
                            Ok(v) => v,
                            Err(_) => {
                                context_refmut.window.alert_with_message(&format!("无法将 ReadableStreamByobReader::read_with_array_buffer_view() 的返回值中的 value 属性转换为 Uint8Array 对象，文件名：{}。程序无法继续运行，若希望重试请刷新页面。", file_name)).unwrap();
                                panic!();
                            },
                        };

                        (value, done)
                    },
                    Err(e) => {
                        context_refmut.window.alert_with_message(&format!("文件流读取失败，文件名：{}。程序无法继续运行，若希望重试请刷新页面。上游错误：{:?}", file_name, e)).unwrap();
                        panic!();
                    }
                };
                let wasm_slice = &mut context_refmut.buffer_data_wasm[0..read_view.0.length() as usize];
                read_view.0.copy_to(wasm_slice);

                // wasm buffer -> crc writer -> compress writer -> transform writer
                if let Err(e) = crc_writer.write_all(wasm_slice) {
                    context_refmut.window.alert_with_message(&format!("Writer 链写入失败，文件名：{}。程序无法继续运行，若希望重试请刷新页面。上游错误：{:?}", file_name, e)).unwrap();
                    panic!();
                }

                // last buffer -> js buffer -> writer
                let last_buffer = crc_writer.last_buffer();
                context_refmut.check_buffer();
                let write_view = js_sys::Uint8Array::new_with_byte_offset_and_length(&context_refmut.buffer_data, 0, last_buffer.len() as u32);
                write_view.copy_from(last_buffer);
                if let Err(e) = wasm_bindgen_futures::JsFuture::from(context_refmut.writer.as_ref().unwrap().write_with_chunk(&write_view)).await {
                    context_refmut.window.alert_with_message(&format!("写入失败，文件名：{}。程序无法继续运行，若希望重试请刷新页面。上游错误：{:?}", file_name, e)).unwrap();
                    panic!();
                }
                context_refmut.bytes_written += write_view.byte_length() as u64;

                // clear buffers in the chain
                crc_writer.clear_buffer();

                if read_view.1 {
                    // flush, and do the copying job on last time
                    if let Err(e) = crc_writer.flush() {
                        context_refmut.window.alert_with_message(&format!("Writer 链刷新失败，文件名：{}。程序无法继续运行，若希望重试请刷新页面。上游错误：{:?}", file_name, e)).unwrap();
                        panic!();
                    }

                    // last buffer -> js buffer -> writer
                    let last_buffer = crc_writer.last_buffer();
                    if last_buffer.len() > 0 {
                        context_refmut.check_buffer();
                        let write_view = js_sys::Uint8Array::new_with_byte_offset_and_length(&context_refmut.buffer_data, 0, last_buffer.len() as u32);
                        write_view.copy_from(last_buffer);
                        if let Err(e) = wasm_bindgen_futures::JsFuture::from(context_refmut.writer.as_ref().unwrap().write_with_chunk(&write_view)).await {
                            context_refmut.window.alert_with_message(&format!("写入失败，文件名：{}。程序无法继续运行，若希望重试请刷新页面。上游错误：{:?}", file_name, e)).unwrap();
                            panic!();
                        }
                        context_refmut.bytes_written += write_view.byte_length() as u64;
                    }

                    break;
                }
            }

            // TODO: write data descriptor

            rust_closure_clone_clone.borrow().take_item.as_ref().unwrap()();
        });
    }));

    // callback - file_entry_error

    let context_clone = Rc::clone(&context);
    context_refmut.callback.file_entry_error = Some(Closure::<dyn Fn(web_sys::DomException)>::new(move |err: web_sys::DomException| {
        let context_ref = context_clone.borrow();

        context_ref.window.alert_with_message(&format!(
            "无法通过 FileSystemFileEntry 对象获取 File 对象，由于程序编写方式的限制，此处无法提供具体的文件名称。程序无法继续执行，如果希望重试的话请刷新页面，上游错误：{:?}",
            err,
        ))
        .unwrap();

        panic!();
    }));

    // callback - directory_get_file_entry_success

    let context_clone = Rc::clone(&context);
    context_refmut.callback.directory_get_file_entry_success = Some(Closure::<dyn Fn(web_sys::FileSystemFileEntry)>::new(move |file_entry: web_sys::FileSystemFileEntry| {
        let context_ref = context_clone.borrow();
        file_entry.file_with_callback_and_callback(
            context_ref.callback.file_entry_success.as_ref().unwrap().as_ref().unchecked_ref(),
            context_ref.callback.file_entry_error.as_ref().unwrap().as_ref().unchecked_ref(),
        );
    }));

    // callback - directory_get_file_entry_error

    let context_clone = Rc::clone(&context);
    context_refmut.callback.directory_get_file_entry_error = Some(Closure::<dyn Fn(web_sys::DomException)>::new(move |err: web_sys::DomException| {
        let context_ref = context_clone.borrow();

        context_ref.window.alert_with_message(&format!(
            "无法通过 FileSystemDirectoryEntry 对象获取 FileSystemFileEntry 对象，由于程序编写方式的限制，此处无法提供具体的文件名称。程序无法继续执行，如果希望重试的话请刷新页面，上游错误：{:?}",
            err,
        ))
        .unwrap();

        panic!();
    }));

    // callback - directory_reader_success

    let context_clone = Rc::clone(&context);
    let rust_closure_clone = Rc::clone(&rust_closure);
    context_refmut.callback.directory_reader_success = Some(Closure::<dyn Fn(js_sys::Array)>::new(move |entries: js_sys::Array| {
        let len = entries.length();
        let mut v = Vec::<web_sys::FileSystemEntry>::with_capacity(len as usize);
        for i in 0..len {
            // TODO: why dyn_into doesn't work?
            // let entry = entries.get(i).dyn_into::<web_sys::FileSystemEntry>().unwrap();
            let entry = entries.get(i).unchecked_into::<web_sys::FileSystemEntry>();
            v.push(entry);
        }
        let entries = v;

        let mut context_refmut = context_clone.borrow_mut();
        context_refmut.unresolved_directories -= 1;
        drop(context_refmut);

        let rust_closure_ref = rust_closure_clone.borrow();
        rust_closure_ref.scan.as_ref().unwrap()(entries);
    }));

    // callback - directory_reader_error

    let context_clone = Rc::clone(&context);
    context_refmut.callback.directory_reader_error = Some(Closure::<dyn Fn(web_sys::DomException)>::new(move |err: DomException| {
        let context_refmut = context_clone.borrow_mut();

        context_refmut.window.alert_with_message(&format!(
            "在遍历用户拖放的文件时遇到了一个无法读取其内容的目录，由于程序编写方式的限制，此处无法提供具体的目录名称。程序无法继续运行，如果希望重试的话请刷新页面，上游错误：{:?}",
            err,
        ))
        .unwrap();

        panic!();
    }));

    // rust_closure - scan

    let context_clone = Rc::clone(&context);
    rust_closure_refmut.scan = Some(Box::new(move |entries: Vec<web_sys::FileSystemEntry>| {
        let mut context_refmut = context_clone.borrow_mut();

        for entry in entries {
            let full_path = entry.full_path();

            if entry.is_file() {
                context_refmut.file_item_list.push(FileItem { path: full_path, is_file: true });
            }
            else if entry.is_directory() {
                context_refmut.file_item_list.push(FileItem { path: full_path.clone(), is_file: false });

                context_refmut.unresolved_directories += 1;

                let directory_entry = entry.unchecked_into::<web_sys::FileSystemDirectoryEntry>();
                let directory_reader = directory_entry.create_reader();
                if let Err(e) = directory_reader.read_entries_with_callback_and_callback(
                    context_refmut.callback.directory_reader_success.as_ref().unwrap().as_ref().unchecked_ref(),
                    context_refmut.callback.directory_reader_error.as_ref().unwrap().as_ref().unchecked_ref(),
                ) {
                    context_refmut.window.alert_with_message(&format!("在遍历用户拖放的文件时，无法读取路径为 {} 目录，程序无法继续运行，若想重试请刷新页面，错误详情：{:?}", full_path, e)).unwrap();
                };
            }
            else {
                context_refmut.window.alert_with_message(&format!("在遍历用户拖放的文件时，路径为 {} 的文件既不是文件也不是目录，程序无法继续运行，若想重试请刷新页面", full_path)).unwrap();
                panic!();
            }
        }

        context_refmut.status_element.set_inner_text(&format!("0/{}", context_refmut.file_item_list.len()));

        if context_refmut.unresolved_directories == 0 {
            context_refmut.file_item_list.sort_by(|a, b| { a.path.cmp(&b.path) });
            context_refmut.window.alert_with_message(&format!("扫描完成，共发现 {} 个文件", context_refmut.file_item_list.len())).unwrap();
        }
    }));

    // rust_closure - take_item

    let context_clone = Rc::clone(&context);
    let rust_closure_clone = Rc::clone(&rust_closure);
    rust_closure_refmut.take_item = Some(Rc::new(move || {
        let mut context_refmut = context_clone.borrow_mut();

        if context_refmut.number_compressed == context_refmut.file_item_list.len() as u64 {
            drop(context_refmut);

            let context_clone_clone = Rc::clone(&context_clone);
            wasm_bindgen_futures::spawn_local(async move {
                let context_ref = context_clone_clone.borrow();
                if let Err(e) = await_promise(context_ref.writer.as_ref().unwrap().close()).await {
                    context_ref.window.alert_with_message(&format!("最终步骤，输出流关闭失败。上游错误：{:?}", e)).unwrap();
                }
            });

            return;
        }

        let old_number_compressed = context_refmut.number_compressed;
        let new_number_compressed = old_number_compressed + 1;

        context_refmut.number_compressed = new_number_compressed;
        let Some(file_item) = context_refmut.file_item_list.get(old_number_compressed as usize)
        else {
            context_refmut.window.alert_with_message(&format!("文件读取失败，索引： {} (0-based)", context_refmut.number_compressed)).unwrap();
            panic!();
        };

        let path = file_item.path.clone();

        let rust_closure_ref = rust_closure_clone.borrow();
        if file_item.is_file {
            drop(context_refmut);
            rust_closure_ref.get_file_entry.as_ref().unwrap()(path);
        }
        else {
            drop(context_refmut);
            rust_closure_ref.process_directory.as_ref().unwrap()(path);
        }
    }));

    // rust_closure - get_file_entry

    let context_clone = Rc::clone(&context);
    rust_closure_refmut.get_file_entry = Some(Box::new(move |path: String| {
        let mut context_refmut = context_clone.borrow_mut();

        let file_header = appnote63::FileHeader::new(path.trim_start_matches('/').to_owned(), context_refmut.bytes_written);
        context_refmut.file_headers.push(file_header);
        context_refmut.current_file_header = Some(context_refmut.file_headers.len() - 1);

        drop(context_refmut);

        let context_clone_clone = Rc::clone(&context_clone);
        wasm_bindgen_futures::spawn_local(async move {
            let mut context_refmut = context_clone_clone.borrow_mut();

            // write "local file header"
            context_refmut.check_buffer();
            let file_header = context_refmut.file_headers.get(context_refmut.current_file_header.unwrap()).unwrap();
            let lfh_view = file_header.write_into_as_lfh(&context_refmut.buffer_header);
            if let Err(e) = await_promise(context_refmut.writer.as_ref().unwrap().write_with_chunk(&lfh_view)).await {
                context_refmut.window.alert_with_message(&format!("\"local file header\" 写入失败，限于程序编写方式，此处无法提供具体的文件名。程序无法继续运行，若希望重试，请刷新页面。上游错误：{:?}", e)).unwrap();
                panic!();
            }
            context_refmut.bytes_written += lfh_view.byte_length() as u64;

            // continue to get the content of the file
            context_refmut.file_system.as_ref().unwrap().root().get_file_with_path_and_options_and_callback_and_callback(
                Some(&path),
                &FileSystemFlags::default(),
                context_refmut.callback.directory_get_file_entry_success.as_ref().unwrap().as_ref().unchecked_ref(),
                context_refmut.callback.directory_get_file_entry_error.as_ref().unwrap().as_ref().unchecked_ref(),
            );
        });
    }));

    // rust_closure - process_directory TODO

    let context_clone = Rc::clone(&context);
    rust_closure_refmut.process_directory = Some(Box::new(move |path: String| {
    }));

    drop(context_refmut);
    drop(rust_closure_refmut);

    if let Err(e) = init_drag_and_drop(Rc::clone(&context), Rc::clone(&rust_closure)) {
        return Err(e);
    }

    if let Err(e) = init_compress_encrypt(Rc::clone(&context), Rc::clone(&rust_closure)) {
        return Err(e);
    }

    mem::forget(context);
    mem::forget(rust_closure);

    Ok("WASM 业务逻辑初始化成功".into())
}

/// Check the existence and type of a specific element.
fn validate_element<T: JsCast>(document: &web_sys::Document, element_id: &str) -> Result<T, ()> {
    use wasm_bindgen::JsCast;

    let Some(element) = document.get_element_by_id(&element_id) else { return Err(()); };
    let Ok(typed_element) = element.dyn_into::<T>() else { return Err(()); };
    Ok(typed_element)
}

/// Bootstrap the scanning procedure in drop event.
fn init_drag_and_drop(context: Rc<RefCell<Context>>, rust_closure: Rc<RefCell<RustClosure>>) -> Result<(), JsValue> {
    let context_refmut = context.borrow_mut();

    let dragover_handler = Closure::<dyn Fn(web_sys::DragEvent)>::new(|ev: web_sys::DragEvent| {
        ev.prevent_default();
        ev.data_transfer().and_then(|data_transfer| Some(data_transfer.set_drop_effect("move")));
    });
    context_refmut.dropping_area_element.add_event_listener_with_callback("dragover", dragover_handler.as_ref().unchecked_ref())?;
    dragover_handler.forget();

    let context_clone = Rc::clone(&context);
    let drop_handler = Closure::<dyn Fn(web_sys::DragEvent)>::new(move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        ev.data_transfer().and_then(|data_transfer| {
            let mut entries = Vec::<web_sys::FileSystemEntry>::new();
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

            let mut context_refmut = context_clone.borrow_mut();
            if let Some(entry) = entries.get(0) {
                context_refmut.file_system = Some(entry.filesystem());
            }
            context_refmut.file_item_list.clear();
            context_refmut.unresolved_directories = 0;
            drop(context_refmut);

            rust_closure.borrow().scan.as_ref().unwrap()(entries);

            Some(())
        });
    });
    context_refmut.dropping_area_element.add_event_listener_with_callback("drop", drop_handler.as_ref().unchecked_ref())?;
    drop_handler.forget();

    Ok(())
}

/// Compress & encrypt
fn init_compress_encrypt(context: Rc<RefCell<Context>>, rust_closure: Rc<RefCell<RustClosure>>) -> Result<(), JsValue> {
    let context_refmut = context.borrow_mut();

    let context_clone = Rc::clone(&context);
    let click_handler = Closure::<dyn Fn(web_sys::PointerEvent)>::new(move |_ev: web_sys::PointerEvent| {
        // use immutable reference to validate current state
        let context_ref = context_clone.borrow();

        let Some(_) = context_ref.file_system.as_ref() else {
            context_ref.window.alert_with_message("文件系统尚未准备完成，请先拖入文件。").unwrap();
            return;
        };
        let file_item_list = &context_ref.file_item_list;
        if file_item_list.len() == 0 {
            context_ref.window.alert_with_message("文件列表为空，请先拖入文件。").unwrap();
            return;
        };
        if context_ref.writer.is_some() {
            context_ref.window.alert_with_message("输出流已存在，请等待当前输出流关闭。").unwrap();
            return;
        }

        // turn to mutable reference
        drop(context_ref);
        let mut context_refmut = context_clone.borrow_mut();

        context_refmut.writer = Some(create_stream_writer());
        context_refmut.number_compressed = 0;
        context_refmut.file_headers.clear();
        context_refmut.bytes_written = 0;

        drop(context_refmut);

        // start point of compress & encrypt procedure
        let rust_closure_ref = rust_closure.borrow();
        let take_item = rust_closure_ref.take_item.as_ref().unwrap();
    });
    context_refmut.compress_encrypt_element.add_event_listener_with_callback("click", click_handler.as_ref().unchecked_ref())?;
    click_handler.forget();

    Ok(())
}

async fn await_promise(promise: js_sys::Promise) -> Result<JsValue, JsValue> {
    wasm_bindgen_futures::JsFuture::from(promise).await
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

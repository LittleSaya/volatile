use std::rc::Rc;

use crate::{alert, prelude::*};

use super::super::{Context, FilePath};

/// Iterate entries tree use BFS, increase the number of unresolved directories when encountering directories (this number will be decreased in next callback).
///
/// The scan stage will complete as soon as the number of unresolved directories reaches zero.
pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.rust_closure.scan.set(Box::new(move |entries: Vec<web_sys::FileSystemEntry>| {
    let context = &context_clone;

    for entry in entries {
      let full_path = entry.full_path();

      if entry.is_file() {
        context.scan_stage.file_path_list.borrow_mut().push(FilePath { path: full_path, is_dir: false });
      }
      else if entry.is_directory() {
        context.scan_stage.file_path_list.borrow_mut().push(FilePath { path: full_path.clone(), is_dir: true });

        *context.scan_stage.unresolved_directories.borrow_mut() += 1;

        let directory_entry = entry.unchecked_into::<web_sys::FileSystemDirectoryEntry>();
        let directory_reader = directory_entry.create_reader();
        if let Err(e) = directory_reader.read_entries_with_callback_and_callback(
          context.callback.FileSystemDirectoryReader_readEntries_success
            .get().as_ref().unwrap_or_else(|| alert::error(context, &format!("在深度遍历用户拖放的文件时，处理路径为 {} 的目录时，发现回调 FileSystemDirectoryReader_readEntries_success 尚未初始化。", full_path)))
            .as_ref().unchecked_ref(),
          context.callback.FileSystemDirectoryReader_readEntries_error
            .get().as_ref().unwrap_or_else(|| alert::error(context, &format!("在深度遍历用户拖放的文件时，处理路径为 {} 的目录时，发现回调 FileSystemDirectoryReader_readEntries_error 尚未初始化。", full_path)))
            .as_ref().unchecked_ref(),
        ) {
          alert::error(context, &format!("在深度遍历用户拖放的文件时，在处理路径为 {} 的目录时，函数 FileSystemDirectoryReader::read_entries_with_callback_and_callback 执行失败。上游错误： {:?}", full_path, e));
        };
      }
      else {
        alert::error(context, &format!("在深度遍历用户拖放的文件时，发现路径为 {} 的文件既不是文件也不是目录。", full_path));
      }
    }

    context.element.status.set_inner_text(&format!("0/{}", context.scan_stage.file_path_list.borrow().len()));

    if *context.scan_stage.unresolved_directories.borrow() == 0 {
      context.scan_stage.file_path_list.borrow_mut().sort_by(|a, b| a.path.cmp(&b.path));
      alert::info(context, &format!("扫描完成，共发现 {} 个文件", context.scan_stage.file_path_list.borrow().len()));
    }
  })) {
    alert::error(context, "重复初始化 rust 闭包 scan");
  };
}

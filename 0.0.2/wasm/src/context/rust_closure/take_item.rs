use std::rc::Rc;

use crate::alert;

use super::super::{Context, FilePath};

/// If the number of compressed files has reached the number of all files, call finish().
///
/// Else, take a file and pass its path to different closures based on file type.
pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.rust_closure.take_item.set(Box::new(move || {
    let context = &context_clone;

    if *context.compress_encrypt_stage.number_compressed.borrow() == context.scan_stage.file_path_list.borrow().len() as u64 {
      context.rust_closure.finish.get().unwrap_or_else(|| alert::error(context, "在 rust 闭包 take_item 中，发现 rust 闭包 finish 尚未初始化。"))();
      return;
    }

    let old_number_compressed = *context.compress_encrypt_stage.number_compressed.borrow();
    let new_number_compressed = old_number_compressed + 1;

    context.compress_encrypt_stage.number_compressed.replace(new_number_compressed);

    let file_path_list_ref = context.scan_stage.file_path_list.borrow();
    let Some(FilePath { path, is_dir}) = file_path_list_ref.get(old_number_compressed as usize) else {
      alert::error(context, &format!("文件读取失败，索引： {} (0-based)", old_number_compressed));
    };

    if *is_dir {
      context.rust_closure.process_directory.get().unwrap_or_else(|| alert::error(context, "在 rust 闭包 take_item 中，发现 rust 闭包 process_directory 尚未初始化。"))(path.clone());
    }
    else {
      context.rust_closure.get_file_entry.get().unwrap_or_else(|| alert::error(context, "在 rust 闭包 take_item 中，发现 rust 闭包 get_file_entry 尚未初始化。"))(path.clone());
    }
  })) {
    alert::error(context, "重复初始化 rust 闭包 take_item");
  };
}

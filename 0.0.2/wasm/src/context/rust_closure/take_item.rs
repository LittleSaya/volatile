use std::rc::Rc;

use crate::{alert, utils};

use super::super::{Context, FilePath};

/// If the number of compressed files has reached the number of all files, close the writer.
///
/// Else, take a file and pass its path to different closures based on its type.
pub fn init(context: &Rc<Context>) {
  let context_clone = Rc::clone(context);
  if let Err(_) = context.rust_closure.take_item.set(Box::new(move || {
    let context = &context_clone;

    if *context.compress_encrypt_stage.number_compressed.borrow() == context.scan_stage.file_path_list.borrow().len() as u64 {
      let context_clone_clone = Rc::clone(&context_clone);
      wasm_bindgen_futures::spawn_local(async move {
        let context = &context_clone_clone;
        if let Err(e) = utils::await_promise(context.compress_encrypt_stage.writer.borrow().as_ref().unwrap_or_else(|| alert::error(context, "在执行最终步骤时，发现 writer 尚未创建。")).close()).await {
          alert::error(context, &format!("在执行最终步骤时，未能正确关闭输出流。上游错误：{:?}", e));
        }
        context.compress_encrypt_stage.writer.replace(None);
      });

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

// Business logic/业务逻辑

import init, { init as init_business } from './wasm_0_0_2.js';

const streamSaver = window.streamSaver;
streamSaver.mitm = 'http://127.0.0.1:5500/mitm.html';
// streamSaver.mitm = 'https://volatile.saya.pw/mitm.html';

function FileItem(
  /** @type {boolean} */ is_dir,
  /** @type {string} */ path,
  /** @type {File} */ file
) {
  this.is_dir = is_dir;
  this.path = path;
  this.file = file;
}

/** @type {FileItem[]} */
let file_item_list = [];

/** @type {HTMLSpanElement} */
let el_number_of_entries = null;

/** @type {HTMLSpanElement} */
let el_number_of_processed_entries = null;

/** @type {boolean} */
let wasm_initialized = false;

document.addEventListener('DOMContentLoaded', async () => {
  try {
    await init();
  }
  catch (err) {
    alert('初始化 wasm 失败：' + JSON.stringify(err));
    return;
  }

  try {
    let res = await init_business(
      'dropping_area',
      'status',
      'compress_encrypt',
      'decrypt'
    );
    alert(JSON.stringify(res));
  }
  catch (err) {
    alert('WASM 运行时错误：' + JSON.stringify(err));
  }
  // if (!(await init_wasm())) return;
  // if (!init_number_of_entries()) return;
  // if (!init_number_of_processed_entries()) return;
  // if (!init_dropping_area()) return;
  // if (!init_compress_encrypt()) return;
  // if (!init_decrypt()) return;
});

async function init_wasm() {
  try {
    await init();
    wasm_initialized = true;
    return true;
  }
  catch (err) {
    wasm_initialized = false;
    alert('WASM 初始化失败，可能需要更新浏览器 (' + JSON.stringify(err) + ')');
    return false;
  }
}

function init_number_of_entries() {
  el_number_of_entries = document.querySelector('span#number_of_entries');
  if (!el_number_of_entries) {
    alert('未找到“文件/文件夹数量”元素');
    return false;
  }
  else {
    el_number_of_entries.innerText = '0';
    return true;
  }
}

function update_number_of_entries() {
  el_number_of_entries.innerText = String(file_item_list.length);
}

function init_number_of_processed_entries() {
  el_number_of_processed_entries = document.querySelector('span#number_of_processed_entries');
  if (!el_number_of_processed_entries) {
    alert('未找到“已处理文件/文件夹数量”元素');
    return false;
  }
  else {
    el_number_of_processed_entries.innerText = '0';
    return true;
  }
}

function update_number_of_processed_entries(processed) {
  el_number_of_processed_entries.innerText = String(processed);
}

function init_dropping_area() {
  /** @type {HTMLButtonElement} */
  let dropping_area = document.querySelector('div#dropping_area');

  if (!dropping_area) {
    alert('未找到“Drop区域”元素');
    return false;
  }

  dropping_area.addEventListener('dragover', (ev) => {
    ev.preventDefault();
    ev.dataTransfer.dropEffect = "move";
  });

  dropping_area.addEventListener('drop', (ev) => {
    ev.preventDefault();
    try {
      /** @type {FileSystemEntry[]} */
      let fs_entry_list = [];
      for (let dt_item of ev.dataTransfer.items) {
        let fs_entry = dt_item.webkitGetAsEntry();
        if (!fs_entry) {
          throw '无法读取文件，可能需要更新浏览器 (DataTransferItem.webkitGetAsEntry returns null)';
        }
        fs_entry_list.push(fs_entry);
      }

      file_item_list = [];
      populate_file_item_list(fs_entry_list);
      file_item_list.sort((a, b) => a.path.localeCompare(b.path));
    }
    catch (err) {
      alert('文件列表读取失败：' + JSON.stringify(err));
    }
  });

  return true;
}

function populate_file_item_list(
  /** @type {FileSystemEntry[]} */ fs_entry_list
) {
  for (let fs_entry of fs_entry_list) {
    if (fs_entry.isFile) {
      /** @type {FileSystemFileEntry} */
      let fs_file_entry = fs_entry;
      fs_file_entry.file(
        file => {
          file_item_list.push(new FileItem(false, fs_file_entry.fullPath, file));
          update_number_of_entries();
        },
        err => {
          throw err;
        }
      );
    }
    else if (fs_entry.isDirectory) {
      /** @type {FileSystemDirectoryEntry} */
      let fs_directory_entry = fs_entry;
      file_item_list.push(new FileItem(true, fs_directory_entry.fullPath, null));
      update_number_of_entries();
      fs_directory_entry.createReader().readEntries(
        populate_file_item_list,
        err => { throw err }
      );
    }
    else {
      throw '文件类型异常：' + fs_entry.fullPath;
    }
  }
}

function init_compress_encrypt() {
  /** @type {HTMLButtonElement} */
  let button = document.querySelector('button#compress_encrypt');

  if (!button) {
    alert('未找到“压缩并加密按钮”元素');
    return false;
  }

  button.addEventListener('click', compress_encrypt);

  return true;
}

async function compress_encrypt() {
  if (!wasm_initialized) {
    alert('WASM 未正确初始化，无法压缩和加密');
    return;
  }
  if (!file_item_list.length) {
    alert('文件列表为空，无法压缩和加密');
    return;
  }

  let context = create_zip_stream_context();

  let file_path_buffer = new ArrayBuffer(1024);
  let file_path_buffer_view = new Uint8Array(file_path_buffer, 0, 1024);
  let text_encoder = new TextEncoder();

  /** @type {WritableStream<Uint8Array>} */
  const output_stream = streamSaver.createWriteStream('请修改此文件名.zip', {
      size: undefined,
      writableStrategy: undefined,
      readableStrategy: undefined
  });
  const writer = output_stream.getWriter();

  let counter = 0;
  try {
    for (let file_item of file_item_list) {
      // encode string to utf-8 in javascript to avoid memory allocation in wasm when
      // converting from JsString to String.
      let path = file_item.path;
      let path_encode_result = text_encoder.encodeInto(path, file_path_buffer_view);
      if (path_encode_result.read !== path.length) {
        throw '无法将文件路径编码为 utf-8 格式，可能是路径过长，编码后的路径长度必须小于 1KB';
      }
      let encoded_path_length = path_encode_result.written;

      if (file_item.is_dir) {
        await push_directory(context, file_path_buffer_view, encoded_path_length, writer);
      }
      else {
        let input_stream = file_item.file.stream();
        let reader = input_stream.getReader({ mode: 'byob' });
        await push_file(context, file_path_buffer_view, encoded_path_length, reader, writer);
      }
      counter += 1;
      update_number_of_processed_entries(counter);
    }

    await finish(context, writer);
  }
  catch (err) {
    alert('压缩或加密失败：' + JSON.stringify(err));
  }
  finally {
    try {
      context.free();
    }
    catch (err) {
      alert('未能正确释放 WASM 上下文对象：' + JSON.stringify(err));
    }

    try {
      await writer.close();
    }
    catch (err) {
      alert('未能正确关闭输出流：' + JSON.stringify(err));
    }
  }
}

function init_decrypt() {

}

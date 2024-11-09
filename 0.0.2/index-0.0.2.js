// Business logic/业务逻辑

import init, * as wasm from './wasm_zip_stream.js';

const streamSaver = window.streamSaver;
streamSaver.mitm = 'https://volatile.saya.pw/mitm.html';

/** @type {boolean} */
let wasm_initialized = false;

/** @type {wasm.Handles} */
let handles = undefined;

/** @type {HTMLSpanElement} */
let scan_progress_element = document.querySelector('span#scan_progress');

/** @type {HTMLSpanElement} */
let compress_progress_element = document.querySelector('span#compress_progress');

/** @type {HTMLSpanElement} */
let average_speed_element = document.querySelector('span#average_speed');

/** @type {HTMLSpanElement} */
let current_speed_element = document.querySelector('span#current_speed');

/** @type {HTMLSpanElement} */
let current_file_element = document.querySelector('span#current_file');

/** @type {HTMLInputElement} */
let compression_level_element = document.querySelector('input#compression_level');

/** @type {HTMLInputElement} */
let compressed_file_name_element = document.querySelector('input#compressed_file_name');

/**
 * @param {wasm.WasmError} e
 */
function translate_wasm_error(e) {
  switch (e.code) {
    case 0x00000001: return `未知的文件类型，错误位置：${e.arg0}`;
    case 0x00000002: return `空文件列表，错误位置：${e.arg0}`;
    case 0x00000003: return `缺少文件系统，错误位置：${e.arg0}`;
    case 0x00000004: return `写入失败，错误位置：${e.arg0}，上游错误：${e.arg1}`;
    case 0x00000005: return `无法获取文件项目，错误位置：${e.arg0}，上游错误：${e.arg1}`;
    case 0x00000006: return `无法获取文件，错误位置：${e.arg0}，上游错误：${e.arg1}`;
    case 0x00000007: return `输入流读取失败，错误位置：${e.arg0}，上游错误：${e.arg1}`;
    case 0x00000009: return `压缩失败，错误位置：${e.arg0}，上游错误：${e.arg1}`;
    case 0x0000000A: return `无法关闭输出流，错误位置：${e.arg0}，上游错误：${e.arg1}`;
    case 0x0000000B: return `文件过多，错误位置：${e.arg0}`;
    case 0x0000000C: return `无法对文件夹执行复原操作，错误位置：${e.arg0}，路径：${e.arg1}`;
    case 0x0000000D: return `错误的压缩等级，错误位置：${e.arg0}，传入的压缩等级：${e.arg1}（正确的压缩等级应该在0~9之间）`;
    case 0x0000000E: return `回调调用失败，错误位置：${e.arg0}，回调名称：${e.arg1}，上游错误：${e.arg2}`;
    case 0x0000000F: return `无法创建输出流，错误位置：${e.arg0}，上游错误：${e.arg1}`;
    case 0x00000010: return `无法对文件夹执行转换操作，错误位置：${e.arg0}，路径：${e.arg1}`;
    default: return `未知错误，错误码：${e.code}，参数0：${e.arg0}，参数1：${e.arg1}，参数2：${e.arg2}，参数3：${e.arg3}`;
  }
}

/**
 * @param {wasm.WasmError} wasm_error
 */
function alert_wasm_error(title, wasm_error) {
  alert(title + '\r\n' + translate_wasm_error(wasm_error));
}

function reset() {
  scan_progress_element.innerText = '0';
  compress_progress_element.innerText = '0 %';
  average_speed_element.innerText = '0 KiB/S';
  current_speed_element.innerText = '0 KiB/S';
  current_file_element.innerText = '?';
}

document.addEventListener('DOMContentLoaded', async () => {
  try {
    if (!wasm_initialized) {
      await init();
      handles = wasm.initialize_context(file_name => {
        /** @type {WritableStream<Uint8Array>} */
        const output_stream = streamSaver.createWriteStream(file_name, {
          size: undefined,
          writableStrategy: undefined,
          readableStrategy: undefined
        });
        const writer = output_stream.getWriter();
        return writer;
      });
      wasm_initialized = true;
    }
    else {
      throw '重复初始化';
    }
  }
  catch (err) {
    alert('初始化 WASM 失败：' + JSON.stringify(err));
    return;
  }

  handles.register_scan_progress(number_of_files => scan_progress_element.innerText = `${number_of_files}`);

  handles.register_compress_progress((number_of_compressed_files, number_of_all_files) => compress_progress_element.innerText = `${(number_of_compressed_files / number_of_all_files * 100).toFixed(2)} % (${number_of_compressed_files}/${number_of_all_files})`);

  handles.register_average_speed((total_bytes_written, total_time_elapsed) => average_speed_element.innerText = `${((total_bytes_written / 1024) / (total_time_elapsed / 1000)).toFixed(2)} KiB/S`);

  handles.register_current_speed((delta_bytes_written, delta_time_elapsed) => current_speed_element.innerText = `${((delta_bytes_written / 1024) / (delta_time_elapsed / 1000)).toFixed(2)} KiB/S`);

  handles.register_current_file(path => current_file_element.innerText = path);

  /** @type {HTMLDivElement} */
  let dropping_area = document.querySelector('div#dropping_area');

  dropping_area.addEventListener('dragover', ev => {
    ev.preventDefault();
    ev.dataTransfer.dropEffect = 'move';
  });

  dropping_area.addEventListener('drop', async ev => {
    ev.preventDefault();

    reset();

    let entries = [];
    for (let item of ev.dataTransfer.items) {
      entries.push(item.webkitGetAsEntry());
    }

    try {
      let number_of_files = await handles.scan(entries);
      scan_progress_element.innerText = `已完成，共 ${number_of_files} 个文件`;
    }
    catch (err) {
      alert_wasm_error('扫描失败', err);
    }
  });

  /** @type {HTMLButtonElement} */
  let transform_only = document.querySelector('button#transform_only');

  transform_only.addEventListener('click', async ev => {
    try {
      await handles.transform('placeholder');
    }
    catch (err) {
      alert_wasm_error('转换失败', err);
    }
  });

  /** @type {HTMLButtonElement} */
  let compress_only = document.querySelector('button#compress_only');

  compress_only.addEventListener('click', async ev => {
    try {
      let compression_level = Number(compression_level_element.value);
      let compressed_file_name = compressed_file_name_element.value ? compressed_file_name_element.value : '请修改文件名.zip';
      await handles.compress(compressed_file_name, compression_level);
    }
    catch (err) {
      alert_wasm_error('压缩失败', err);
    }
  });

  /** @type {HTMLButtonElement} */
  let compress_transform = document.querySelector('button#compress_transform');

  compress_transform.addEventListener('click', async ev => {
    try {
      let compression_level = Number(compression_level_element.value);
      let compressed_file_name = compressed_file_name_element.value ? compressed_file_name_element.value : '请修改文件名.zip';
      await handles.compress_transform(compressed_file_name, compression_level, 'placeholder');
    }
    catch (err) {
      alert_wasm_error('压缩转换失败', err);
    }
  });

  /** @type {HTMLButtonElement} */
  let recover = document.querySelector('button#recover');

  recover.addEventListener('click', async ev => {
    try {
      await handles.recover('placeholder');
    }
    catch (err) {
      alert_wasm_error('复原失败', err);
    }
  });
});

// Business logic/业务逻辑

import init, * as wasm from './wasm_0_0_2.js';

const streamSaver = window.streamSaver;
streamSaver.mitm = 'http://127.0.0.1:8080/mitm.html';
// streamSaver.mitm = 'https://volatile.saya.pw/mitm.html';

/** @type {boolean} */
let wasm_initialized = false;

/** @type {boolean} */
let wasm_business_initialized = false;

document.addEventListener('DOMContentLoaded', async () => {
  try {
    if (!wasm_initialized) {
      await init();
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

  // await main();
  await test_1();
});

async function main() {
  try {
    if (!wasm_business_initialized) {
      await wasm.main(
        'dropping_area',
        'status',
        'compress_encrypt',
        'decrypt'
      );
      wasm_business_initialized = true;
    }
    else {
      throw '重复初始化';
    }
  }
  catch (err) {
    alert('WASM 运行时错误：' + JSON.stringify(err));
  }

  window.create_stream_writer = function () {
    /** @type {WritableStream<Uint8Array>} */
    const output_stream = streamSaver.createWriteStream('请修改此文件名.zip', {
      size: undefined,
      writableStrategy: undefined,
      readableStrategy: undefined
    });
    const writer = output_stream.getWriter();
    return writer;
  }
}

async function test_1() {
  try {
    console.log('test 1: create an arraybuffer in rust, and test when it will become invalid');
    await wasm.test_1();
  }
  catch (err) {
    alert('WASM 运行时错误：' + JSON.stringify(err));
  }
}

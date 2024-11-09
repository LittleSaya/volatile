/* tslint:disable */
/* eslint-disable */
/**
 * @param {Function} create_writer
 * @returns {Handles}
 */
export function initialize_context(create_writer: Function): Handles;
export class FileHeader {
  free(): void;
}
export class Handles {
  free(): void;
  /**
   * Do a deep scan on input entries.
   *
   * # Parameters
   *
   * * `entries` - MUST be an array of `FileSystemEntry`
   *
   * # Returns
   *
   * - resolve: number of scanned entries
   * - reject: a `WasmError` object
   * @param {Array<any>} entries
   * @returns {Promise<any>}
   */
  scan(entries: Array<any>): Promise<any>;
  /**
   * Compress scanned entries.
   *
   * # Parameters
   *
   * * `output_file_name` - name of output zip file
   * * `compression_level` - an integer, minimal 0, maximum 9
   *
   * # Returns
   *
   * - resolve: undefined
   * - reject: a `WasmError` object
   * @param {string} output_file_name
   * @param {number} compression_level
   * @returns {Promise<any>}
   */
  compress(output_file_name: string, compression_level: number): Promise<any>;
  /**
   * Compress scanned entries and transform output bytes.
   *
   * # Parameters
   *
   * * `output_file_name` - name of output zip file
   * * `compression_level` - an integer, minimal 0, maximum 9
   * * `transform_script` - NOT IMPLEMENTED
   *
   * # Returns
   *
   * - resolve: undefined
   * - reject: a `WasmError` object
   * @param {string} output_file_name
   * @param {number} compression_level
   * @param {string} transform_script
   * @returns {Promise<any>}
   */
  compress_transform(output_file_name: string, compression_level: number, transform_script: string): Promise<any>;
  /**
   * Just transform the input file, only one file could be accepted.
   *
   * # Parameters
   *
   * * `transform_script` - NOT IMPLEMENTED
   *
   * # Returns
   *
   * - resolve: undefined
   * - reject: a `WasmError` object
   * @param {string} _transform_script
   * @returns {Promise<any>}
   */
  transform(_transform_script: string): Promise<any>;
  /**
   * Recover the input file, only one file could be accepted.
   *
   * # Parameters
   *
   * * `transform_script` - NOT IMPLEMENTED
   *
   * # Returns
   *
   * - resolve: undefined
   * - reject: a `WasmError` object
   * @param {string} _transform_script
   * @returns {Promise<any>}
   */
  recover(_transform_script: string): Promise<any>;
  /**
   * Register "scan_progress" callback.
   *
   * # Parameters
   *
   * * `callback` - a function like `(number_of_scanned_entries: number) => {}`
   * @param {Function} callback
   */
  register_scan_progress(callback: Function): void;
  /**
   * Register "compress_progress" callback.
   *
   * # Parameters
   *
   * * `callback` - a function like `(number_of_compressed_files: number, number_of_all_files: number) => {}`
   * @param {Function} callback
   */
  register_compress_progress(callback: Function): void;
  /**
   * Register "average_speed" callback.
   *
   * # Parameters
   *
   * * `callback` - a function like `(total_bytes_written: number, total_time_elapsed: number) => {}`
   * @param {Function} callback
   */
  register_average_speed(callback: Function): void;
  /**
   * Register "current_speed" callback.
   *
   * # Parameters
   *
   * * `callback` - a function like `(delta_bytes_written: number, delta_time_elapsed: number) => {}`
   * @param {Function} callback
   */
  register_current_speed(callback: Function): void;
  /**
   * Register "current_current_filespeed" callback.
   *
   * # Parameters
   *
   * * `callback` - a function like `(path: string) => {}`
   * @param {Function} callback
   */
  register_current_file(callback: Function): void;
}
export class WasmError {
  free(): void;
  arg0: string;
  arg1: string;
  arg2: string;
  arg3: string;
  code: number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wasmerror_free: (a: number, b: number) => void;
  readonly __wbg_get_wasmerror_code: (a: number) => number;
  readonly __wbg_set_wasmerror_code: (a: number, b: number) => void;
  readonly __wbg_get_wasmerror_arg0: (a: number, b: number) => void;
  readonly __wbg_set_wasmerror_arg0: (a: number, b: number, c: number) => void;
  readonly __wbg_get_wasmerror_arg1: (a: number, b: number) => void;
  readonly __wbg_set_wasmerror_arg1: (a: number, b: number, c: number) => void;
  readonly __wbg_get_wasmerror_arg2: (a: number, b: number) => void;
  readonly __wbg_set_wasmerror_arg2: (a: number, b: number, c: number) => void;
  readonly __wbg_get_wasmerror_arg3: (a: number, b: number) => void;
  readonly __wbg_set_wasmerror_arg3: (a: number, b: number, c: number) => void;
  readonly __wbg_handles_free: (a: number, b: number) => void;
  readonly handles_scan: (a: number, b: number) => number;
  readonly handles_compress: (a: number, b: number, c: number, d: number) => number;
  readonly handles_compress_transform: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
  readonly handles_transform: (a: number, b: number, c: number) => number;
  readonly handles_recover: (a: number, b: number, c: number) => number;
  readonly handles_register_scan_progress: (a: number, b: number) => void;
  readonly handles_register_compress_progress: (a: number, b: number) => void;
  readonly handles_register_average_speed: (a: number, b: number) => void;
  readonly handles_register_current_speed: (a: number, b: number) => void;
  readonly handles_register_current_file: (a: number, b: number) => void;
  readonly initialize_context: (a: number) => number;
  readonly __wbg_fileheader_free: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5b3f9cd6c38a6389: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h1c265f1720677f49: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly wasm_bindgen__convert__closures__invoke2_mut__h76ed5b8c1aa880a0: (a: number, b: number, c: number, d: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;

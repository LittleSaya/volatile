use web_sys::{js_sys, wasm_bindgen};
use wasm_bindgen::prelude::*;

use crate::constant::{VERSION_MADE_BY, VERSION_NEEDED_TO_EXTRACT};

/// # FileHeader
///
/// This type contains necessary fields for both "local file header" and
/// "central directory header" because they have many similar fields.
#[wasm_bindgen]
pub struct FileHeader {
  version_made_by                     : u16, // CDH only
  version_needed_to_extract           : u16, // LFH & CDH
  general_purpose_bit_flag            : u16, // LFH & CDH
  compression_method                  : u16, // LFH & CDH
  last_mod_file_time                  : u16, // LFH & CDH
  last_mod_file_date                  : u16, // LFH & CDH
  crc_32                              : u32, // LFH & CDH
  compressed_size                     : u32, // LFH & CDH
  uncompressed_size                   : u32, // LFH & CDH
  compressed_size_u64                 : u64, // zip64
  uncompressed_size_u64               : u64, // zip64
  file_name_length                    : u16, // LFH & CDH
  disk_number_start                   : u16, // CDH only
  internal_file_attributes            : u16, // CDH only
  external_file_attributes            : u32, // CDH only
  relative_offset_of_local_header     : u32, // CDH only
  lfh_pos                             : u64,
  file_name                           : Vec<u8>,
}

impl FileHeader {
  pub fn new(file_name: String, lfh_pos: u64) -> Self {
    // File name could be a relative path, which MUST not contain a driver or device
    // letter, or a leading slash. All slashes MUST be '/'.
    // Ref 4.4.17
    let file_name: Vec<u8> = file_name.into();

    Self {
      // Appnote version 6.3 is referenced while writing these codes.
      // Ref 4.4.2 & 4.4.3
      version_made_by                 : VERSION_MADE_BY,

      // The newest feature I need is "zip64", which requires a version number of 4.5
      // Ref 4.4.2 & 4.4.3
      version_needed_to_extract       : VERSION_NEEDED_TO_EXTRACT,

      // (starting from 0)
      // Bit 1 & 2 are kept zero because they stand for "Normal compression option".
      // Bit 3 is set because this will enable the "data descriptor".
      // Bit 11 is set because this indicates the file name and comment are utf-8 encoded.
      // Ref 4.4.4
      general_purpose_bit_flag: 0b0000_1000_0000_1000_u16,

      // Method 8 is Deflate.
      // Ref 4.4.5
      compression_method: 8_u16,

      // TODO
      last_mod_file_date: 0_u16,

      // TODO
      last_mod_file_time: 0_u16,

      // The actual crc is stored in "data descriptor".
      // Ref 4.4.4
      //
      // This field will be set after finishing compressing.
      crc_32: 0_u32,

      // LFH: Because this program always uses "zip64", these 2 fields are set to 0xFFFFFFFF
      // here, the "zero" values will be placed in extra data, which indicates that the
      // actual compressed size and uncompressed size are stored in "data descriptor".
      //
      // CDH: Both set to 0xFFFFFFFF because of "zip64".
      compressed_size: 0xFFFFFFFF_u32,
      uncompressed_size: 0xFFFFFFFF_u32,

      // These fields store the real file size, and will be set after finishing compressing.
      compressed_size_u64: 0_u64,
      uncompressed_size_u64: 0_u64,

      file_name_length: file_name.len() as u16,

      // "zip64", in extra data.
      disk_number_start: 0xFFFF_u16,

      // TODO: I don't understand the description of this field.
      // Ref 4.4.14
      internal_file_attributes: 0_u16,

      // TODO: I don't know how to use this field.
      // Ref 4.4.15
      external_file_attributes: 0_u32,

      // "zip64", in extra data.
      relative_offset_of_local_header: 0xFFFFFFFF_u32,

      lfh_pos,

      file_name,
    }
  }

  /// Pass in a wasm buffer, return a Uint8Array viewing this buffer.
  pub fn write_into_as_lfh(&self, buffer: &mut Vec<u8>, array_buffer: &js_sys::ArrayBuffer) -> js_sys::Uint8Array {
    let FileHeader {
      version_needed_to_extract,
      general_purpose_bit_flag,
      compression_method,
      last_mod_file_time,
      last_mod_file_date,
      crc_32,
      compressed_size,
      uncompressed_size,
      file_name_length,
      file_name,
      ..
    } = self;

    let local_file_header_signature = 0x04034b50_u32.to_le_bytes();
    let version_needed_to_extract = version_needed_to_extract.to_le_bytes();
    let general_purpose_bit_flag = general_purpose_bit_flag.to_le_bytes();
    let compression_method = compression_method.to_le_bytes();
    let last_mod_file_time = last_mod_file_time.to_le_bytes();
    let last_mod_file_date = last_mod_file_date.to_le_bytes();
    let crc_32 = crc_32.to_le_bytes();
    let compressed_size = compressed_size.to_le_bytes();
    let uncompressed_size = uncompressed_size.to_le_bytes();
    let file_name_length = file_name_length.to_le_bytes();

    // Currently, this program will always use "Zip64 extended information extra field", its size is 20 bytes in LFH
    const EXTRA_FIELD_LENGTH: usize = 20;
    let mut extra_field: [u8; EXTRA_FIELD_LENGTH] = [0; EXTRA_FIELD_LENGTH];
    extra_field[0..2].copy_from_slice(&0x0001_u16.to_le_bytes()); // Tag for this "extra" block type
    extra_field[2..4].copy_from_slice(&16_u16.to_le_bytes()); // Size of this "extra" block
    extra_field[4..12].copy_from_slice(&0_u64.to_le_bytes()); // Original uncompressed file size
    extra_field[12..20].copy_from_slice(&0_u64.to_le_bytes()); // Size of compressed data

    buffer.clear();

    buffer.extend_from_slice(&local_file_header_signature);
    buffer.extend_from_slice(&version_needed_to_extract);
    buffer.extend_from_slice(&general_purpose_bit_flag);
    buffer.extend_from_slice(&compression_method);
    buffer.extend_from_slice(&last_mod_file_time);
    buffer.extend_from_slice(&last_mod_file_date);
    buffer.extend_from_slice(&crc_32);
    buffer.extend_from_slice(&compressed_size);
    buffer.extend_from_slice(&uncompressed_size);
    buffer.extend_from_slice(&file_name_length);
    buffer.extend_from_slice(&(EXTRA_FIELD_LENGTH as u16).to_le_bytes());
    buffer.extend_from_slice(file_name);
    buffer.extend_from_slice(&extra_field);

    let view = js_sys::Uint8Array::new_with_byte_offset_and_length(array_buffer, 0, buffer.len() as u32);
    view.copy_from(buffer);

    view
  }

  pub fn write_into_as_cdh(&self, buffer: &mut Vec<u8>) {
    // Currently, this program will always use "Zip64 extended information extra field", its size is 32 bytes in CDH
    const EXTRA_FIELD_LENGTH: usize = 32;
    let mut extra_field: [u8; EXTRA_FIELD_LENGTH] = [0; EXTRA_FIELD_LENGTH];
    extra_field[0..2].copy_from_slice(&0x0001_u16.to_le_bytes()); // Tag for this "extra" block type
    extra_field[2..4].copy_from_slice(&28_u16.to_le_bytes()); // Size of this "extra" block
    extra_field[4..12].copy_from_slice(&self.uncompressed_size_u64.to_le_bytes()); // Original uncompressed file size
    extra_field[12..20].copy_from_slice(&self.compressed_size_u64.to_le_bytes()); // Size of compressed data
    extra_field[20..28].copy_from_slice(&self.lfh_pos.to_le_bytes()); // Offset of local header record
    extra_field[28..32].copy_from_slice(&0_u32.to_le_bytes()); // Number of the disk on which this file starts

    // central file header signature
    buffer.extend_from_slice(&0x02014b50_u32.to_le_bytes());
    buffer.extend_from_slice(&self.version_made_by.to_le_bytes());
    buffer.extend_from_slice(&self.version_needed_to_extract.to_le_bytes());
    buffer.extend_from_slice(&self.general_purpose_bit_flag.to_le_bytes());
    buffer.extend_from_slice(&self.compression_method.to_le_bytes());
    buffer.extend_from_slice(&self.last_mod_file_time.to_le_bytes());
    buffer.extend_from_slice(&self.last_mod_file_date.to_le_bytes());
    buffer.extend_from_slice(&self.crc_32.to_le_bytes());
    buffer.extend_from_slice(&self.compressed_size.to_le_bytes());
    buffer.extend_from_slice(&self.uncompressed_size.to_le_bytes());
    buffer.extend_from_slice(&self.file_name_length.to_le_bytes());
    buffer.extend_from_slice(&(EXTRA_FIELD_LENGTH as u16).to_le_bytes());
    buffer.extend_from_slice(&0_u16.to_le_bytes()); // file comment length
    buffer.extend_from_slice(&self.disk_number_start.to_le_bytes());
    buffer.extend_from_slice(&self.internal_file_attributes.to_le_bytes());
    buffer.extend_from_slice(&self.external_file_attributes.to_le_bytes());
    buffer.extend_from_slice(&self.relative_offset_of_local_header.to_le_bytes());
    buffer.extend_from_slice(&self.file_name);
    buffer.extend_from_slice(&extra_field);
    // buffer.extend_from_slice(&self.file_comment); // no file comment
  }

  pub fn set_crc_32(&mut self, crc32: u32) {
    self.crc_32 = crc32;
  }

  pub fn set_compressed_size_u64(&mut self, size: u64) {
    self.compressed_size_u64 = size;
  }

  pub fn set_uncompressed_size_u64(&mut self, size: u64) {
    self.uncompressed_size_u64 = size;
  }
}
